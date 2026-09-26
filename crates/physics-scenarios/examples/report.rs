//! Prints the engine's own numbers for both scenarios, next to the hand
//! calculation for each. Anything quoted in the write-up should come from here
//! rather than from memory.
//!
//! Run with: cargo run -p physics-scenarios --example report

use physics_core::analysis::percent_difference;
use physics_core::scenario::Scenario;
use physics_core::sim::Simulation;
use physics_scenarios::{InclineScenario, SpringScenario};

fn main() {
    spring_report();
    println!();
    incline_report();
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
