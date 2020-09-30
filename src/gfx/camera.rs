use cgmath::
{
    Vector3,
    vec3,
    Matrix3,
    Matrix4,
    Quaternion,
    Rad,
    Deg,
    Rotation3,
    ElementWise,
    InnerSpace,
};

pub struct Camera
{
    eye_pos: Vector3<f32>,
    looking_at: Vector3<f32>,

    world_up: Vector3<f32>,
    world_forward: Vector3<f32>,
    world_right: Vector3<f32>,

    orientation: Quaternion<f32>,
    translation: Vector3<f32>,

    zoom: f32,
    zoom_min: f32,
    zoom_max: f32
}

impl Camera
{
    // TODO: These constructor function names are just place holders so that I can write the functions,
    //  they're horrible for actual function names
    // TODO: Cache results from functions that return computed values

    pub fn from_three(eye_pos: Vector3<f32>, looking_at: Vector3<f32>, world_up: Vector3<f32>) -> Camera
    {
        let world_forward = looking_at.sub_element_wise(eye_pos);
        Camera::from_all(
            eye_pos,
            looking_at,
            world_up,
            world_forward,
            world_forward.cross(world_up)
        )
    }

    pub fn from_all(eye_pos: Vector3<f32>, looking_at: Vector3<f32>, world_up: Vector3<f32>, world_forward: Vector3<f32>, world_right: Vector3<f32>) -> Camera
    {
        Camera
        {
            eye_pos,
            looking_at,
            world_up,
            world_forward,
            world_right,
            orientation: Quaternion::from_axis_angle(world_up, Rad(0.0)),
            translation: vec3(0.0, 0.0, 0.0),
            zoom: 0.0,
            zoom_min: 0.0,
            zoom_max: 0.0
        }
    }

    /// Move World

    pub fn set_world_translation(&mut self, translation: Vector3<f32>)
    {
        self.translation = translation;
    }

    pub fn move_world_lat(&mut self, delta: f32)
    {
        self.translation += delta * self.world_right;
    }

    pub fn move_world_vert(&mut self, delta: f32)
    {
        self.translation += self.world_forward * delta;
    }

    pub fn move_world_long(&mut self, delta: f32)
    {
        self.translation += self.world_forward * delta;
    }

    pub fn move_world(&mut self, delta: Vector3<f32>)
    {
        self.move_world_lat(delta.x);
        self.move_world_vert(delta.y);
        self.move_world_long(delta.z);
    }

    /// Rotate World

    pub fn rotate_world_yaw(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * Quaternion::from_axis_angle(self.world_up, Deg(theta))).normalize();
    }

    pub fn rotate_world_pitch(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * Quaternion::from_axis_angle(self.world_right, Deg(theta))).normalize();
    }

    /// Move Camera

    pub fn set_cam_position(&mut self, pos: Vector3<f32>)
    {
        self.translation = pos * -1.0;
    }

    pub fn move_cam_lat(&mut self, delta: f32)
    {
        self.translation += self.orientation * (delta * self.world_right);
    }

    pub fn move_cam_vert(&mut self, delta: f32)
    {
        self.translation += self.orientation * (delta * self.world_up);
    }

    pub fn move_cam_long(&mut self, delta: f32)
    {
        self.translation += self.orientation * (delta * self.world_forward);
    }

    pub fn move_cam(&mut self, delta: Vector3<f32>)
    {
        self.move_cam_lat(delta.x);
        self.move_cam_vert(delta.y);
        self.move_cam_long(delta.z);
    }

    pub fn move_cam_lat_locked(&mut self, delta: f32)
    {
        let delta_lat = self.orientation * (delta * self.world_right);
        self.translation += vec3(delta_lat.x, 0.0, delta_lat.z);
    }

    pub fn move_cam_vert_locked(&mut self, delta: f32)
    {
        self.move_world_vert(-delta);
    }

    pub fn move_cam_long_locked(&mut self, delta: f32)
    {
        let delta_lat = self.orientation * (delta * self.world_forward);
        self.translation += vec3(delta_lat.x, 0.0, delta_lat.z);
    }

    pub fn move_cam_locked(&mut self, delta: Vector3<f32>)
    {
        self.move_cam_lat_locked(delta.x);
        self.move_cam_vert_locked(delta.y);
        self.move_cam_long_locked(delta.z);
    }

    /// Rotate Camera

    pub fn rotate_cam(&mut self, orientation: Quaternion<f32>)
    {
        self.orientation = (self.orientation * orientation).normalize();
    }

    pub fn rotate_cam_yaw(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * Quaternion::from_axis_angle(self.orientation * self.world_up, Deg(theta))).normalize();
    }

    pub fn rotate_cam_pitch(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * Quaternion::from_axis_angle(self.orientation * self.world_right, Deg(theta))).normalize();
    }

    pub fn rotate_cam_roll(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * Quaternion::from_axis_angle(self.orientation * self.world_forward, Deg(theta))).normalize();
    }


    pub fn set_orientation(&mut self, orientation: Quaternion<f32>)
    {
        self.orientation = orientation;
    }

    pub fn reset_orientation(&mut self)
    {
        self.orientation = Quaternion::from_axis_angle(vec3(0.0, 0.0, 0.0), Rad(0.0));
    }

    /// Zoom

    pub fn zoom(&mut self, delta: f32)
    {
        self.eye_pos += delta * self.world_forward;
    }

    pub fn reset_zoom(&mut self)
    {
        self.eye_pos -= self.zoom * self.world_forward;
    }

    pub fn set_zoom(&mut self, zoom: f32)
    {
        self.reset_zoom();
        self.eye_pos += zoom * self.world_forward;
        self.zoom = zoom;
    }

    pub fn set_min_zoom(&mut self, zoom_min: f32)
    {
        self.zoom_min = zoom_min;
        if self.zoom < self.zoom_min
        {
            self.zoom = self.zoom_min;
        }
    }

    pub fn set_max_zoom(&mut self, zoom_max: f32)
    {
        self.zoom_max = zoom_max;
        if self.zoom > self.zoom_max
        {
            self.zoom = zoom_max;
        }
    }

    /// Getters

    pub fn get_base_eye_pos(&self) -> Vector3<f32>
    {
        self.eye_pos
    }

    pub fn get_world_eye_pos(&self) -> Vector3<f32>
    {
        (self.orientation * (self.eye_pos - (self.zoom * self.world_forward * 2.0))) + self.translation
    }

    pub fn get_looking_at(&self) -> Vector3<f32>
    {
        self.looking_at
    }

    pub fn get_world_up(&self) -> Vector3<f32>
    {
        self.world_up
    }

    pub fn get_world_forward(&self) -> Vector3<f32>
    {
        self.world_forward
    }

    pub fn get_world_right(&self) -> Vector3<f32>
    {
        self.world_right
    }

    pub fn get_world_translation(&self) -> Vector3<f32>
    {
        self.translation
    }


    pub fn get_eye_pos(&self) -> Vector3<f32>
    {
        -1.0 * self.get_world_eye_pos()
    }

    pub fn get_cam_pos(&self) -> Vector3<f32>
    {
        -1.0 * self.translation
    }

    pub fn get_orientation(&self) -> Quaternion<f32>
    {
        self.orientation
    }

    pub fn get_zoom(&self) -> f32
    {
        self.zoom
    }

    pub fn view_matrix(&self) -> Matrix4<f32>
    {
        let view = Matrix3::look_at(self.looking_at - self.eye_pos, self.world_up) * Into::<Matrix3<f32>>::into(self.orientation);
        let mut view: Matrix4<f32> = view.into();
        view[2] = view[0] * self.translation[0] + view[1] * self.translation[1] + view[2];
        view
    }
}