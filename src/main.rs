use bevy::{color::palettes::css::ORANGE_RED, math::FloatPow, prelude::*};
use chacha20::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use bevy::{
  camera_controller::free_camera::{
    FreeCamera, FreeCameraPlugin, FreeCameraState, VerticalMovementAxis,
  },
  prelude::*,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, (interact_bodies, integrate))
    .add_plugins(FreeCameraPlugin)
    .add_plugins((CameraPlugin, CameraSettingsPlugin))
    .run();
}

const GRAVITY_CONSTANT: f32 = 0.001;

#[derive(Component)]
struct Ground;

#[derive(Component, Default)]
struct Mass(f32);
#[derive(Component, Default)]
struct Acceleration(Vec3);
#[derive(Component, Default)]
struct LastPos(Vec3);

#[derive(Bundle, Default)]
struct BodyBundle {
  mesh: Mesh3d,
  material: MeshMaterial3d<StandardMaterial>,
  mass: Mass,
  last_pos: LastPos,
  acceleration: Acceleration,
}

fn setup(
  time: Res<Time<Fixed>>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(20., 1., 20.))),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::srgb(0.3, 0.5, 0.3),
      cull_mode: None,
      ..default()
    })),
    Ground,
  ));

  let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);

  let mesh = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
  let color_range = 0.5..1.0;
  let vel_range = -0.5..0.5;

  let radius: f32 = rng.random_range(0.1..0.7);
  let mass_value = FloatPow::cubed(radius) * 10.;

  let position = Vec3::new(
    rng.random_range(-1.0..1.0),
    rng.random_range(-1.0..1.0),
    rng.random_range(-1.0..1.0),
  )
  .normalize()
    * ops::cbrt(rng.random_range(0.2f32..1.0))
    * 15.;

  commands.spawn((
    BodyBundle {
      mesh: Mesh3d(mesh.clone()),
      material: MeshMaterial3d(materials.add(Color::srgb(
        rng.random_range(color_range.clone()),
        rng.random_range(color_range.clone()),
        rng.random_range(color_range.clone()),
      ))),
      mass: Mass(mass_value),
      acceleration: Acceleration(Vec3::ZERO),
      last_pos: LastPos(
        position
          - Vec3::new(
            rng.random_range(vel_range.clone()),
            rng.random_range(vel_range.clone()),
            rng.random_range(vel_range.clone()),
          ) * time.timestep().as_secs_f32(),
      ),
    },
    Transform {
      translation: position,
      scale: Vec3::splat(radius),
      ..default()
    },
  ));

  commands.spawn((
    DirectionalLight::default(),
    Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
  ));
}

struct CameraPlugin;
impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, spawn_camera);
  }
}

fn spawn_camera(mut commands: Commands) {
  commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 1.0, 0.0).looking_to(Vec3::X, Vec3::Y),
    FreeCamera {
      sensitivity: 0.2,
      friction: 25.0,
      walk_speed: 3.0,
      run_speed: 9.0,
      ..default()
    },
  ));
}

struct CameraSettingsPlugin;
impl Plugin for CameraSettingsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(PostStartup, spawn_text)
      .add_systems(Update, (update_camera_settings, update_text));
  }
}

#[derive(Component)]
struct InfoText;

fn spawn_text(mut commands: Commands, free_camera_query: Query<&FreeCamera>) {
  commands.spawn((
    Node {
      position_type: PositionType::Absolute,
      top: px(-16),
      left: px(12),
      ..default()
    },
    children![Text::new(format!(
      "{}",
      free_camera_query.single().unwrap(),
    ))],
  ));
  commands.spawn((
    Node {
      position_type: PositionType::Absolute,
      bottom: px(12),
      left: px(12),
      ..default()
    },
    children![Text::new(concat![
      "Z/X: decrease/increase sensitivity\n",
      "C/V: decrease/increase friction\n",
      "F/G: decrease/increase scroll factor\n",
      "B: enable/disable controller\n",
      "T: world/local vertical movement"
    ]),],
  ));

  // Mutable text marked with component
  commands.spawn((
    Node {
      position_type: PositionType::Absolute,
      top: px(12),
      right: px(12),
      ..default()
    },
    children![(InfoText, Text::new(""))],
  ));
}

fn update_camera_settings(
  mut camera_query: Query<(&mut FreeCamera, &mut FreeCameraState)>,
  input: Res<ButtonInput<KeyCode>>,
) {
  let (mut free_camera, mut free_camera_state) = camera_query.single_mut().unwrap();

  if input.pressed(KeyCode::KeyZ) {
    free_camera.sensitivity = (free_camera.sensitivity - 0.005).max(0.005);
  }
  if input.pressed(KeyCode::KeyX) {
    free_camera.sensitivity += 0.005;
  }
  if input.pressed(KeyCode::KeyC) {
    free_camera.friction = (free_camera.friction - 0.2).max(0.0);
  }
  if input.pressed(KeyCode::KeyV) {
    free_camera.friction += 0.2;
  }
  if input.pressed(KeyCode::KeyF) {
    free_camera.scroll_factor = (free_camera.scroll_factor - 0.02).max(0.02);
  }
  if input.pressed(KeyCode::KeyG) {
    free_camera.scroll_factor += 0.02;
  }
  if input.just_pressed(KeyCode::KeyB) {
    free_camera_state.enabled = !free_camera_state.enabled;
  }
  if input.just_pressed(KeyCode::KeyT) {
    free_camera.vertical_movement_axis = match free_camera.vertical_movement_axis {
      VerticalMovementAxis::World => VerticalMovementAxis::Local,
      VerticalMovementAxis::Local => VerticalMovementAxis::World,
    };
  }
}

fn update_text(
  mut text_query: Query<&mut Text, With<InfoText>>,
  camera_query: Query<(&FreeCamera, &FreeCameraState)>,
) {
  let mut text = text_query.single_mut().unwrap();

  let (free_camera, free_camera_state) = camera_query.single().unwrap();

  text.0 = format!(
    "Enabled: {},\nSensitivity: {:.03}\nFriction: {:.01}\nScroll factor: {:.02}\nWalk Speed: {:.02}\nRun Speed: {:.02}\nSpeed: {:.02}",
    free_camera_state.enabled,
    free_camera.sensitivity,
    free_camera.friction,
    free_camera.scroll_factor,
    free_camera.walk_speed * free_camera_state.speed_multiplier,
    free_camera.run_speed * free_camera_state.speed_multiplier,
    free_camera_state.velocity.length(),
  );
}

fn interact_bodies(mut query: Query<(&Mass, &GlobalTransform, &mut Acceleration)>) {
  let mut iter = query.iter_combinations_mut();
  while let Some(
    [
      (Mass(m1), transform1, mut acc1),
      (Mass(m2), transform2, mut acc2),
    ],
  ) = iter.fetch_next()
  {
    let delta = transform2.translation() - transform1.translation();
    let distance_sq: f32 = delta.length_squared();

    let f = GRAVITY_CONSTANT / distance_sq;
    let force_unit_mass = delta * f;
    acc1.0 += force_unit_mass * *m2;
    acc2.0 -= force_unit_mass * *m1;
  }
}

fn integrate(time: Res<Time>, mut query: Query<(&mut Acceleration, &mut Transform, &mut LastPos)>) {
  let dt_sq = time.delta_secs() * time.delta_secs();
  for (mut acceleration, mut transform, mut last_pos) in &mut query {
    // verlet integration
    // x(t+dt) = 2x(t) - x(t-dt) + a(t)dt^2 + O(dt^4)

    let new_pos = transform.translation * 2.0 - last_pos.0 + acceleration.0 * dt_sq;
    acceleration.0 = Vec3::ZERO;
    last_pos.0 = transform.translation;
    transform.translation = new_pos;
  }
}
