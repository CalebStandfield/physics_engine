//! Prints the engine's own numbers for every scenario, next to the hand
//! calculation for each. Anything quoted in the write-up should come from here
//! rather than from memory.
//!
//! Run with: cargo run -p physics-scenarios --example report

use physics_core::analysis::percent_difference;
use physics_core::scenario::Scenario;
use physics_core::sim::Simulation;
use physics_scenarios::{InclineScenario, PendulumScenario, SpringScenario};

fn main() {
    spring_report();
    println!();
    incline_report();
    println!();
    pendulum_report();
}

fn spring_report() {
    let mut s = SpringScenario::new();
    s.set_param("mass", 0.25).unwrap();
    s.set_param("stiffness", 20.0).unwrap();
    s.set_param("initial_displacement", 0.05).unwrap();

    let predicted = s.ideal_period();
    let stretch = s.equilibrium_stretch();

    let mut sim = Simulation::new(Box::new(s));
    sim.run_for(20.0 * predicted);
    let measured = sim.measured_period().unwrap();

    println!("Spring: 0.250 kg on a 20.0 N/m spring");
    println!("  equilibrium stretch  x0 = m g / k      = {stretch:.6} m");
    println!("  predicted period     T  = 2 pi sqrt(m/k) = {predicted:.6} s");
    println!("  simulated period     over {} cycles      = {measured:.6} s", sim.cycles());
    println!(
        "  percent difference                        = {:.4} %",
        percent_difference(measured, predicted)
    );
}

fn incline_report() {
    let mut s = InclineScenario::new();
    s.set_param("mass", 1.50).unwrap();
    s.set_param("angle_deg", 30.0).unwrap();
    s.set_param("mu_static", 0.25).unwrap();
    s.set_param("mu_kinetic", 0.20).unwrap();
    s.set_param("length", 3.0).unwrap();

    let normal = s.normal_force();
    let along = s.gravity_parallel();
    let friction = s.kinetic_friction();
    let predicted = s.ideal_sliding_accel();

    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-5);
    sim.run_for(0.5);
    let st = sim.state();
    // Constant acceleration from rest: a = 2 x / t^2.
    let measured = 2.0 * st.x / (st.t * st.t);

    println!("Incline: 1.50 kg on a 30.0 deg ramp, mu_k = 0.20");
    println!("  gravity along slope  m g sin(theta)     = {along:.6} N");
    println!("  normal force         N = m g cos(theta) = {normal:.6} N");
    println!("  kinetic friction     mu_k N             = {friction:.6} N");
    println!("  predicted accel      g(sin - mu_k cos)  = {predicted:.6} m/s^2");
    println!("  simulated accel      2 x / t^2 at t = {:.3} s = {measured:.6} m/s^2", st.t);
    println!(
        "  percent difference                        = {:.4} %",
        percent_difference(measured, predicted)
    );
}

fn pendulum(pivot: f64, release_deg: f64) -> PendulumScenario {
    let mut s = PendulumScenario::new();
    s.set_param("mass", 0.15).unwrap();
    s.set_param("rod_length", 1.0).unwrap();
    s.set_param("pivot_offset", pivot).unwrap();
    s.set_param("initial_angle_deg", release_deg).unwrap();
    s
}

/// Period the engine measures over a run of many cycles, at a fine step.
fn simulated_period(s: PendulumScenario) -> f64 {
    let span = 20.0 * s.ideal_period();
    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-4);
    sim.run_for(span);
    sim.measured_period().unwrap()
}

fn pendulum_report() {
    let base = pendulum(0.5, 5.0);
    println!("Pendulum: 0.150 kg, 1.00 m stick, released at 5 deg");
    println!(
        "  shortest-period pivot  d_min = L / sqrt(12)       = {:.6} m",
        base.shortest_period_pivot()
    );
    println!(
        "  shortest period        T_min = 2 pi sqrt(2 d / g) = {:.6} s",
        base.shortest_period()
    );
    println!("  d (m)   T_0 small-angle   T exact at 5 deg   simulated   diff vs exact");
    for d in [0.05, 0.10, 0.20, base.shortest_period_pivot(), 0.30, 0.40, 0.50] {
        let s = pendulum(d, 5.0);
        let ideal = s.ideal_period();
        let exact = s.amplitude_period().unwrap();
        let measured = simulated_period(s);
        println!(
            "  {d:.4}  {ideal:.6} s        {exact:.6} s         {measured:.6} s  {:.4} %",
            percent_difference(measured, exact)
        );
    }

    println!("  Pivot at the end, wider releases:");
    println!("  release   T_0 small-angle   T exact   simulated   T_0 off by");
    for release in [10.0, 30.0, 60.0, 90.0, 150.0] {
        let s = pendulum(0.5, release);
        let ideal = s.ideal_period();
        let exact = s.amplitude_period().unwrap();
        let measured = simulated_period(s);
        println!(
            "  {release:>5.0} deg  {ideal:.6} s        {exact:.6} s  {measured:.6} s  {:.4} %",
            percent_difference(ideal, measured)
        );
    }

    // Energy loss on, at what the checkbox sets. Largest angle seen in each
    // second of the run, so the decay reads as a list.
    let mut s = pendulum(0.5, 10.0);
    let on = match physics_scenarios::control("energy_loss").widget {
        physics_core::params::Widget::Switch { on } => on,
        physics_core::params::Widget::Slider => unreachable!("energy_loss is a switch"),
    };
    s.set_param("energy_loss", on).unwrap();
    let b = s.damping();
    let floor = -s.params().mass * s.params().gravity * s.pivot_distance();
    let start = s.initial_state();
    let e0 = s.derived(&start).iter().find(|d| d.key == "mechanical_energy").unwrap().value;
    let mut sim = Simulation::new(Box::new(s));
    sim.set_dt(1e-4);
    println!("  Energy loss on, {on} %/s (b = -I ln(1 - p/100) = {b:.6} N m s), pivot at the end, released at 10 deg:");
    for second in 1..=10 {
        let mut peak: f64 = 0.0;
        while sim.state().t < second as f64 {
            sim.step();
            peak = peak.max(sim.state().x.abs());
        }
        let e = sim.snapshot().derived_value("mechanical_energy").unwrap();
        println!(
            "  t = {second:>2} s  largest angle {:.3} deg  swing energy left {:.2} %  lost {:.6} J (ledger {:.6} J)",
            peak.to_degrees(),
            100.0 * (e - floor) / (e0 - floor),
            e0 - e,
            sim.energy_lost()
        );
    }
}
