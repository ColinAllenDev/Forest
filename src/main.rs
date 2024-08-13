use bevy::prelude::*;
use bevy::app::App;
use bevy::utils::default;
use bevy::time::Time;

mod plugins;
use plugins::obj_loader::ObjLoaderPlugin;

mod camera;
use camera::orbit_camera::{pan_orbit_camera, PanOrbitCameraBundle, PanOrbitState};

// Main function
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ObjLoaderPlugin));
    app.add_systems(Startup, (spawn_mesh, spawn_sun, spawn_camera))
        .add_systems(Update, pan_orbit_camera.run_if(
            any_with_component::<PanOrbitState>)
        )
    .run();
}

fn spawn_mesh(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_handle: Handle<Mesh> = asset_server.load("bear.obj");
    let texture_handle: Handle<Image> = asset_server.load("bear.png");

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.0,
        unlit: true,
        ..default()
    });

    // Spawn single OBJ model
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            ..default()
        },
        Spin,
    ));
}

fn spawn_sun(mut commands: Commands) {
    // Praise the Sun
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 5.0, 1.0)),
        ..default()
    });
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();

    // The camera is positioned by changing the components data
    // Transform would get overwritten
    camera.state.center = Vec3::new(1.0, 2.0, 3.0);
    camera.state.radius = 50.0;
    camera.state.pitch = 15.0f32.to_radians();
    camera.state.yaw = 30.0f32.to_radians();
    commands.spawn(camera);
}

#[derive(Component)]
struct Spin;
fn spin(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.1 * time.delta_seconds());
    }
}
