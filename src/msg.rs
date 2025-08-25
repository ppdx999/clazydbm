/// Messages the root understands.
pub enum AppMsg {
    Quit,
    // ConnectionSelected(ConfigConnection),
    // Dashboard(DashboardMsg), // mapped child
    Error(String),
    // ... async completions, timers, etc.
}
