//! End-to-end checks: the driver, the scenarios and the analysis together.
//!
//! The contract tests at the bottom run against every scenario in the
//! registry, so a new scenario inherits them for free.

use physics_core::analysis::percent_difference;
use physics_core::integrator;
use physics_core::scenario::Scenario;
use physics_core::sim::Simulation;
use physics_core::state::State;
use physics_scenarios::registry;
use physics_scenarios::{InclineScenario, SpringScenario};

fn spring(mass: f64, stiffness: f64, pull: f64) -> SpringScenario {
    let mut s = SpringScenario::new();
    s.set_param("mass", mass).unwrap();
    s.set_param("stiffness", stiffness).unwrap();
    s.set_param("initial_displacement", pull).unwrap();
    s
}

#[test]
fn measured_spring_period_matches_the_predicted_one() {
    let s = spring(0.25, 20.0, 0.05);
    let predicted = s.ideal_period();
    let mut sim = Simulation::new(Box::new(s));
    sim.run_for(20.0 * predicted);

    let measured = sim.measured_period().expect("should have timed cycles");
    assert!(sim.cycles() >= 19, "only {} cycles", sim.cycles());
    let diff = percent_difference(measured, predicted);
    assert!(diff < 1.0, "{measured} s vs {predicted} s, {diff}% apart");
}

#[test]
fn period_does_not_depend_on_amplitude() {
    let small = {
        let mut sim = Simulation::new(Box::new(spring(0.25, 20.0, 0.01)));
        sim.run_for(10.0);
        sim.measured_period().unwrap()
    };
    let large = {
        let mut sim = Simulation::new(Box::new(spring(0.25, 20.0, 0.2)));
        sim.run_for(10.0);
        sim.measured_period().unwrap()
    };
    assert!(percent_difference(small, large) < 0.5);
}

#[test]
fn quadrupling_the_mass_doubles_the_period() {
    let light = {
        let mut sim = Simulation::new(Box::new(spring(0.25, 20.0, 0.05)));
        sim.run_for(15.0);
        sim.measured_period().unwrap()
    };
    let heavy = {
        let mut sim = Simulation::new(Box::new(spring(1.0, 20.0, 0.05)));
        sim.run_for(30.0);
        sim.measured_period().unwrap()
    };
    assert!(percent_difference(heavy, 2.0 * light) < 1.0);
}

#[test]
fn every_integrator_agrees_on_the_period_at_a_small_step() {
    let predicted = spring(0.25, 20.0, 0.05).ideal_period();
    for integ in integrator::all() {
        let id = integ.id();
        let mut sim = Simulation::new(Box::new(spring(0.25, 20.0, 0.05)));
        sim.set_integrator(integ);
        sim.set_dt(1e-5);
        sim.run_for(10.0 * predicted);
        let measured = sim.measured_period().unwrap();
        assert!(
            percent_difference(measured, predicted) < 0.5,
            "{id}: {measured} s vs {predicted} s"
        );
    }
}

#[test]
fn simulated_incline_acceleration_matches_the_hand_calculation() {
    let mut s = InclineScenario::new();
    s.set_param("angle_deg", 30.0).unwrap();
    s.set_param("mu_kinetic", 0.20).unwrap();
    s.set_param("mu_static", 0.25).unwrap();
    s.set_param("length", 3.0).unwrap();
    let predicted = s.ideal_sliding_accel();

    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-5);
    sim.run_for(0.5);

    // Constant acceleration from rest, so a = 2 x / t^2.
    let st = sim.state();
    let measured = 2.0 * st.x / (st.t * st.t);
    assert!(
        percent_difference(measured, predicted) < 0.1,
        "{measured} vs {predicted} m/s^2"
    );
}

#[test]
fn friction_accounts_for_the_missing_energy() {
    let mut s = InclineScenario::new();
    s.set_param("angle_deg", 35.0).unwrap();
    s.set_param("mu_kinetic", 0.25).unwrap();
    s.set_param("mu_static", 0.30).unwrap();
    s.set_param("length", 2.0).unwrap();
    let mass = s.params().mass;
    let gravity = s.params().gravity;
    let drop_per_meter = s.angle().sin();

    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-5);
    sim.run_for(0.8);

    let st = sim.state();
    let released = mass * gravity * drop_per_meter * st.x;
    let kinetic = st.kinetic_energy(mass);
    let lost = sim.energy_lost();
    // Energy from the drop either shows up as motion or goes into friction.
    assert!(
        percent_difference(released, kinetic + lost) < 0.5,
        "released {released} J, kinetic {kinetic} J, lost {lost} J"
    );
}

#[test]
fn reset_restores_the_starting_state() {
    for id in registry::ids() {
        let mut sim = Simulation::new(registry::create(id).unwrap());
        let start = sim.state();
        sim.run_for(2.0);
        sim.reset();
        assert_eq!(sim.state(), start, "{id} did not reset");
        assert_eq!(sim.steps(), 0);
        assert_eq!(sim.energy_lost(), 0.0);
        assert_eq!(sim.recorder().len(), 1);
    }
}

#[test]
fn no_scenario_blows_up_under_any_integrator() {
    for id in registry::ids() {
        for integ in integrator::all() {
            let integ_id = integ.id();
            let mut sim = Simulation::new(registry::create(id).unwrap());
            sim.set_integrator(integ);
            sim.run_for(10.0);
            assert!(sim.state().is_finite(), "{id} + {integ_id} diverged");
            let snap = sim.snapshot();
            assert!(snap.net_force.is_finite());
            assert!(snap.accel.is_finite());
            for d in &snap.derived {
                assert!(d.value.is_finite(), "{id}.{} is not finite", d.key);
            }
        }
    }
}

#[test]
fn every_scenario_draws_forces_that_match_its_net_force() {
    for id in registry::ids() {
        let s = registry::create(id).unwrap();
        for state in [
            s.initial_state(),
            State::new(0.3, s.initial_state().x + 0.1, 0.7),
            State::new(0.6, s.initial_state().x - 0.1, -0.7),
        ] {
            let frame = s.frame(&state);
            let drawn = frame.net_force().component_along(frame.axis);
            let scalar = s.net_force(state.x, state.v, state.t);
            assert!(
                (drawn - scalar).abs() < 1e-9,
                "{id}: drew {drawn} N, computed {scalar} N"
            );
            assert!((frame.axis.len() - 1.0).abs() < 1e-12, "{id} axis not unit");
        }
    }
}

#[test]
fn parameter_ranges_are_all_survivable() {
    // Pairs, not one slider at a time: the combinations are where the stiff
    // cases live (a light mass together with heavy damping, say).
    for id in registry::ids() {
        let schema = registry::create(id).unwrap().schema();
        for (i, first) in schema.iter().enumerate() {
            for second in schema.iter().skip(i + 1) {
                for a in [first.min, first.max] {
                    for b in [second.min, second.max] {
                        let mut s = registry::create(id).unwrap();
                        s.set_param(first.key, a).unwrap();
                        s.set_param(second.key, b).unwrap();
                        let mut sim = Simulation::new(s);
                        sim.run_for(0.5);
                        assert!(
                            sim.state().is_finite(),
                            "{id} diverged at {} = {a}, {} = {b}",
                            first.key,
                            second.key
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn recording_is_thinned_and_ordered() {
    let mut sim = Simulation::new(Box::new(spring(0.25, 20.0, 0.05)));
    sim.set_dt(0.001);
    sim.set_recorder(physics_core::record::Recorder::new(500, 10));
    sim.run_for(1.0);

    let samples = sim.recorder().to_vec();
    assert_eq!(samples.len(), 101, "100 kept steps plus the initial state");
    assert!(samples.windows(2).all(|w| w[1].t >= w[0].t), "out of order");
    assert_eq!(sim.recorder().to_flat().len(), samples.len() * 3);
}

#[test]
fn a_block_at_rest_never_gets_pushed_backwards_by_friction() {
    // mu_k above mu_s is unusual but the sliders allow it. Friction on a
    // stationary block can cancel the drive, not reverse it.
    let mut s = InclineScenario::new();
    s.set_param("angle_deg", 30.0).unwrap();
    s.set_param("mu_static", 0.10).unwrap();
    s.set_param("mu_kinetic", 1.50).unwrap();
    s.set_param("initial_position", 1.0).unwrap();
    assert!(!s.holds_at_rest(), "static friction should be too weak here");
    assert!(
        s.net_force(1.0, 0.0, 0.0) >= 0.0,
        "downhill gravity must not produce an uphill net force"
    );

    let mut sim = Simulation::new(Box::new(s));
    sim.run_for(2.0);
    assert!(sim.state().x >= 1.0 - 1e-9, "block crept uphill");
    assert!(sim.state().v >= -1e-9, "block picked up uphill velocity");
}

#[test]
fn stiff_and_heavily_damped_springs_stay_finite_at_the_default_step() {
    // A light mass on heavy damping is stiff enough to blow up plain Euler at
    // the default step, so the driver has to subdivide.
    for (mass, damping, stiffness) in [
        (0.01, 20.0, 0.1),
        (0.01, 20.0, 500.0),
        (0.01, 0.0, 500.0),
        (20.0, 20.0, 500.0),
    ] {
        for integ in integrator::all() {
            let integ_id = integ.id();
            let mut s = SpringScenario::new();
            s.set_param("mass", mass).unwrap();
            s.set_param("damping", damping).unwrap();
            s.set_param("stiffness", stiffness).unwrap();

            let mut sim = Simulation::new(Box::new(s));
            sim.set_integrator(integ);
            sim.run_for(3.0);
            assert!(
                sim.state().is_finite(),
                "m={mass} b={damping} k={stiffness} diverged under {integ_id}"
            );
        }
    }
}

#[test]
fn subdividing_does_not_change_the_simulated_clock() {
    let mut s = SpringScenario::new();
    s.set_param("mass", 0.01).unwrap();
    s.set_param("stiffness", 500.0).unwrap();
    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(0.01);
    assert!(sim.subdivisions() > 1, "this setup should need subdividing");

    sim.step_n(100);
    assert_eq!(sim.steps(), 100);
    assert!((sim.state().t - 1.0).abs() < 1e-12, "clock drifted");
}

#[test]
fn stopping_at_the_end_of_the_ramp_is_booked_as_energy_lost() {
    let mut s = InclineScenario::new();
    s.set_param("angle_deg", 40.0).unwrap();
    s.set_param("mu_static", 0.10).unwrap();
    s.set_param("mu_kinetic", 0.05).unwrap();
    s.set_param("length", 1.0).unwrap();
    let mass = s.params().mass;
    let gravity = s.params().gravity;
    let drop = s.angle().sin() * 1.0;

    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-5);
    sim.run_for(2.0);
    assert_eq!(sim.state().v, 0.0, "block should be stopped at the end");

    // Everything gravity released over the ramp ended up in the loss ledger.
    let released = mass * gravity * drop;
    assert!(
        percent_difference(sim.energy_lost(), released) < 0.5,
        "lost {} J of {released} J released",
        sim.energy_lost()
    );
}

#[test]
fn a_damped_spring_oscillates_at_its_damped_period() {
    let mut s = SpringScenario::new();
    s.set_param("mass", 0.25).unwrap();
    s.set_param("stiffness", 20.0).unwrap();
    s.set_param("damping", 0.5).unwrap();
    s.set_param("initial_displacement", 0.05).unwrap();
    let undamped = s.ideal_period();
    let damped = s.damped_period().unwrap();
    assert!(damped > undamped, "damping should stretch the period");

    let mut sim = Simulation::new(Box::new(s));
    sim.run_for(10.0 * damped);
    let measured = sim.measured_period().unwrap();
    assert!(
        percent_difference(measured, damped) < 1.0,
        "{measured} s vs {damped} s"
    );
}

// The drawing hints have to agree with the geometry: the box turns with the
// slope, and "up out of the surface" is a unit vector square to the slope.
#[test]
fn the_incline_pose_follows_the_slope() {
    for angle_deg in [0.0, 25.0, 60.0, 89.0] {
        let mut s = InclineScenario::new();
        s.set_param("angle_deg", angle_deg).unwrap();
        let pose = s.frame(&s.initial_state()).pose;
        let axis = s.axis();

        assert!((pose.angle + angle_deg.to_radians()).abs() < 1e-12);
        assert!((pose.support.len() - 1.0).abs() < 1e-12);
        assert!(pose.support.dot(axis).abs() < 1e-12);
        // Out of the surface, not into it.
        assert!(pose.support.y > 0.0);
    }
}

#[test]
fn a_hanging_spring_has_no_pose() {
    let s = spring(0.5, 30.0, 0.05);
    let pose = s.frame(&s.initial_state()).pose;
    assert_eq!(pose.angle, 0.0);
    assert_eq!(pose.support, physics_core::math::Vec2::ZERO);
}
