use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{DefaultPlugins, app::{App, Plugin, Startup, Update}, asset::Assets, camera::Camera3d, camera_controller::free_camera::{FreeCamera, FreeCameraPlugin}, color::{Color, LinearRgba}, ecs::{component::Component, query::With, resource::Resource, schedule::IntoScheduleConfigs, system::{Commands, Query, Res, ResMut}}, light::PointLight, math::{Vec2, Vec3, VectorSpace, primitives::{Circle, Plane3d, Rectangle, Sphere}}, mesh::{Mesh, Mesh3d}, pbr::{MeshMaterial3d, StandardMaterial}, time::{Time, Timer, TimerMode}, transform::components::Transform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(10.0, 10.0)))),
        MeshMaterial3d(materials.add(Color::from(LinearRgba::new(1.0, 1.0, 1.0, 0.7)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..Default::default()
        }
    ));
}