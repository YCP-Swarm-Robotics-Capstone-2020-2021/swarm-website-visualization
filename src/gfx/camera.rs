use cgmath::
{
    Vector3,
    vec3,
    Vector4,
    vec4,
    Point3,
    Matrix3,
    Matrix4,
    Quaternion,
    Rad,
    Deg,
    Rotation3,
    ElementWise,
    InnerSpace,
};

#[derive(Debug, Copy, Clone, PartialEq)]
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
    /// Important notes about the naming conventions for functions:
    ///
    /// Anything regarding the "world" doesn't perform operations from the
    /// camera's view. Manipulating the world is essentially an
    /// inverse camera manipulation. So moving the world "left" makes the
    /// camera seem as if its moving "right"
    ///
    /// Anything regarding the "camera", performs operations from the
    /// camera's view
    ///
    /// *_lat() is lateral movement, i.e. left and right
    /// *_vert() is vertical movement, i.e. up and down
    /// *_long() is longitudinal movement, i.e. forward and back
    /// yaw rotation is rotating around the camera's vertical axis
    ///     i.e. "looking" left/right
    /// pitch rotation is rotating around the camera's lateral/horizontal axis
    ///     i.e. "looking" up/down
    /// roll rotation is rotating around the camera's longitudinal axis
    ///     i.e. tilting your head left/right
    ///
    /// *_locked() operations do not effect the y component.
    ///     i.e. "locking" the camera to its current "height".
    ///     This is good for stuff like a character walking around, as the
    ///     camera can look up/down while staying at the same height

    // TODO: Cache results from functions that return computed values

    /// Creates a new camera from the given eye coordinates
    /// This uses the given parameters to calculate the other necessary
    /// world coordinates
    ///
    /// `eye_pos` is the position of the camera's eye
    /// `looking_at` is the direction that the eye is looking
    /// `world_up` is "up" direction of the world from the camera's view
    pub fn from_eye(eye_pos: Vector3<f32>, looking_at: Vector3<f32>, world_up: Vector3<f32>) -> Camera
    {
        let world_forward = looking_at.sub_element_wise(eye_pos);
        Camera::from_eye_and_world(
            eye_pos,
            looking_at,
            world_up,
            world_forward,
            world_forward.cross(world_up)
        )
    }

    /// Creates a new camera from the given eye and world coordinates
    ///
    /// `eye_pos` is the position of the camera's eye
    /// `looking_at` is the direction that the eye is looking
    /// `world_up` is the "up" direction of the world from the camera's view
    /// `world_forward` is the "forward" direction of the world from the
    ///     camera's view
    /// `world_right` is the "right" direction of the world from the
    ///     camera's view
    pub fn from_eye_and_world(eye_pos: Vector3<f32>, looking_at: Vector3<f32>, world_up: Vector3<f32>, world_forward: Vector3<f32>, world_right: Vector3<f32>) -> Camera
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
            zoom_max: f32::MAX
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
        self.zoom += delta;
        if self.zoom > self.zoom_max
        {
            self.zoom = self.zoom_max;
        }
        else if self.zoom < self.zoom_min
        {
            self.zoom = self.zoom_min;
        }
    }

    pub fn set_zoom(&mut self, zoom: f32)
    {
        self.zoom =
            if zoom > self.zoom_max
            {
                self.zoom_max
            }
            else if zoom < self.zoom_min
            {
                self.zoom_min
            }
            else
            {
                zoom
            };
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

    /// Get the camera's base eye position
    pub fn get_eye_pos(&self) -> Vector3<f32>
    {
        self.eye_pos
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

    pub fn get_zoomed_eye_pos(&self) -> Vector3<f32>
    {
        self.eye_pos + (self.zoom * self.world_forward)
    }

    /// Get the position of the camera's eye within the world
    pub fn get_world_eye_pos(&self) -> Vector3<f32>
    {
        (self.orientation * (self.get_zoomed_eye_pos() - (self.zoom * self.world_forward * 2.0))) + self.translation
    }

    /// Get the position of the camera within the world adjusted to be from
    /// the camera's view
    pub fn get_world_pos_adjusted(&self) -> Vector3<f32>
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
        let eye_pos = self.get_zoomed_eye_pos();
        let mut view = Matrix4::look_at(Point3::new(eye_pos.x, eye_pos.y, eye_pos.z), Point3::new(self.looking_at.x, self.looking_at.y, self.looking_at.z), self.world_up)
            * Into::<Matrix4<f32>>::into(self.orientation);
        crate::log_s(format!("{:?}", view));
        view[3] = self.translation.extend(1.0);
        view
    }
}

#[cfg(test)]
mod tests
{
    use crate::gfx::camera::Camera;
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

    const TEST_CAM: Camera =
        Camera
        {
            eye_pos: vec3(0.0, 0.0, 0.0),
            looking_at: vec3(0.0, 0.0, 0.1),

            world_up: vec3(0.0, 1.0, 0.0),
            world_forward: vec3(0.0, 0.0, 0.1),
            world_right: vec3(-0.1, 0.0, 0.0),

            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            translation: vec3(0.0, 0.0, 0.0),

            zoom: 0.0,
            zoom_min: 0.0,
            zoom_max: 0.0
        };

    #[test]
    fn test_from_eye()
    {
        let cam = Camera::from_eye(
            TEST_CAM.eye_pos,
            TEST_CAM.looking_at,
            TEST_CAM.world_up
        );

        assert_eq!(TEST_CAM, cam);

    }
    #[test]
    fn test_from_eye_and_world()
    {
        let cam = Camera::from_eye_and_world(
            TEST_CAM.eye_pos,
            TEST_CAM.looking_at,
            TEST_CAM.world_up,
            TEST_CAM.world_forward,
            TEST_CAM.world_right
        );

        assert_eq!(TEST_CAM, cam);
    }
}