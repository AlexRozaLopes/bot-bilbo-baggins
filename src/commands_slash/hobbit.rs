use serenity::builder::CreateApplicationCommand;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("hobbit")
        .description("Use este comando por sua conta em risco! (Vc foi avisado!)")
}

pub fn run() -> String {
    "Infelizmente este comando ainda esta em desenvolvimento ;-;".to_string()
}
