use bevy::{
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder, NotShadowCaster}, prelude::*
};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Position(i32, i32, i32);

#[derive(Resource)]
struct UpdateTimer(Timer);

pub struct InitPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (camera_fog_setup, scene_terrain_setup),
         )
        .run();
}

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UpdateTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Update, (set_position, get_position).chain());
    }
}

fn get_position(time: Res<Time>, mut timer: ResMut<UpdateTimer>, query: Query<&Position, With<Player>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for pos in &query {
            println!("Position: {}, {}, {}", pos.0, pos.1, pos.2);
        }
    }
}

fn set_position(mut query: Query<&mut Position, With<Player>>) {
    for mut pos in &mut query {
        pos.0 = 1;
        pos.1 = 1;
        pos.2 = 1;
    }
}

fn camera_fog_setup(mut commands: Commands) {
    commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-1.0, 0.1, 1.0)
                    .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                ..default()
            },
            FogSettings {
                color: Color::srgba(0.35, 0.48, 0.66, 1.0),
                directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
                directional_light_exponent: 30.0,
                falloff: FogFalloff::from_visibility_colors(
                    // Distance (in world units up) to which objects retain visiblity (>= 5% contrast)
                    15.0,
                    // Atmospheric Extinction Color
                    // (after light is lost due to absorption by atmospheric particles.)
                    Color::srgb(0.35, 0.5, 0.66),
                    // Atmospheric Inscattering Color
                    // (light gained due to scattering from the sun)
                    Color::srgb(0.8, 0.844, 1.0)
                ),
            },
    ));
}

fn scene_terrain_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    // Configure a properly scaled cascade shadow map for this scene
    // (defaults too large, mesh units in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Praise the Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.5, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });


    // Terrain
    // TERRAIN GENERATION GOES HERE

    // Sky
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Srgba::hex("888888").unwrap().into(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(20.0)),
            ..default()
        },
        NotShadowCaster
    ));
}

fn add_players(mut commands: Commands) {
    commands.spawn((Player, Position(0, 0, 0)));
}



