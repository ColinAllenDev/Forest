use bevy::prelude::*;
use bevy::app::App;
use bevy::utils::default;
use bevy::time::Time;

mod plugins;
use plugins::obj_loader::ObjLoaderPlugin;

// Main function
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ObjLoaderPlugin));
    app.add_systems(Startup, (load, setup))
    .add_systems(Update, spin)
    .run();
}

fn load(
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

fn setup(mut commands: Commands) {
    // Praise the Sun
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 5.0, 1.0)),
        ..default()
    });
    // Camera
    const ZOOM_FACTOR: f32 = 4.0;
    let translation_vector = Vec3::new(1.0, 1.0 * ZOOM_FACTOR, 1.0 * ZOOM_FACTOR);
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(translation_vector)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct Spin;
fn spin(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.1 * time.delta_seconds());
    }
}
