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
    .add_plugins(FreeCameraPlugin)
    .add_plugins((CameraPlugin, CameraSettingsPlugin))
    .run();
}

#[derive(Component)]
struct Ground;

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn((
    Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
    MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    Ground,
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
