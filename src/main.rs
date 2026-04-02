use bevy::{DefaultPlugins, app::{App, Plugin, Startup, Update}, ecs::{component::Component, query::With, resource::Resource, schedule::IntoScheduleConfigs, system::{Commands, Query, Res, ResMut}}, time::{Time, Timer, TimerMode}};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Alpha".into())));
    commands.spawn((Person, Name("Beta".into())));
    commands.spawn((Person, Name("Gamma".into())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query {
            println!("hello {}!", name.0);
        }
    }
}

fn update_people(query: Query<&mut Name, With<Person>>) {
    for mut name in query {
        if name.0 == "Alpha" {
            name.0 = name.0.to_uppercase();
        }
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app        
            .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Update, (update_people, greet_people).chain())
            .add_systems(Startup, add_people);
    }
}