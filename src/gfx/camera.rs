extern crate nalgebra_glm as glm;

use self::glm::Quat;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera
{
    eye_pos: glm::Vec3,
    looking_at: glm::Vec3,

    world_up: glm::Vec3,
    world_forward: glm::Vec3,
    world_right: glm::Vec3,

    orientation: glm::Quat,
    translation: glm::Vec3,

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
    pub fn from_eye(eye_pos: glm::Vec3, looking_at: glm::Vec3, world_up: glm::Vec3) -> Camera
    {
        let world_forward = looking_at.sub_element_wise(eye_pos.clone());
        Camera::from_eye_and_world(
            eye_pos,
            looking_at,
            world_up,
            world_forward,
            world_forward.cross(world_up.clone())
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
    pub fn from_eye_and_world(eye_pos: glm::Vec3, looking_at: glm::Vec3, world_up: glm::Vec3, world_forward: glm::Vec3, world_right: glm::Vec3) -> Camera
    {
        Camera
        {
            eye_pos,
            looking_at,
            world_up: world_up.normalize(),
            world_forward: world_forward.normalize(),
            world_right: world_right.normalize(),
            orientation: glm::quat_angle_axis(0.0, &world_up.normalize()),//Quaternion::from_axis_angle(world_up.normalize(), Rad(0.0)),
            translation: glm::vec3(0.0, 0.0, 0.0),
            zoom: 0.0,
            zoom_min: 0.0,
            zoom_max: f32::MAX
        }
    }

    /// Move World

    pub fn set_world_translation(&mut self, translation: glm::Vec3)
    {
        self.translation = translation;
    }

    pub fn move_world_lat(&mut self, delta: f32)
    {
        self.translation += delta * &self.world_right;
    }

    pub fn move_world_vert(&mut self, delta: f32)
    {
        self.translation += &self.world_up * delta;
    }

    pub fn move_world_long(&mut self, delta: f32)
    {
        self.translation += &self.world_forward * delta;
    }

    pub fn move_world(&mut self, delta: glm::Vec3)
    {
        self.move_world_lat(delta.x);
        self.move_world_vert(delta.y);
        self.move_world_long(delta.z);
    }

    /// Rotate World

    pub fn rotate_world_yaw(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * glm::quat_angle_axis(glm::radians(&glm::vec1(theta)).x, &self.world_up.clone())).normalize();
        //self.orientation = (self.orientation * Quaternion::from_axis_angle(self.world_up, Deg(theta))).normalize();
    }

    pub fn rotate_world_pitch(&mut self, theta: f32)
    {
        self.orientation = (self.orientation * glm::quat_angle_axis(glm::radians(&glm::vec1(theta)).x, &self.world_right.clone())).normalize();
        //self.orientation = (self.orientation * Quaternion::from_axis_angle(self.world_right, Deg(theta))).normalize();
    }

    /// Move Camera

    pub fn set_cam_position(&mut self, pos: glm::Vec3)
    {
        self.translation = pos * -1.0;
    }

    pub fn move_cam_lat(&mut self, delta: f32)
    {
        self.translation += (self.orientation * (&self.world_right * delta));
    }

    pub fn move_cam_vert(&mut self, delta: f32)
    {
        self.translation += self.orientation * (&self.world_up * delta);
    }

    pub fn move_cam_long(&mut self, delta: f32)
    {
        self.translation += self.orientation * (&self.world_forward * delta);
    }

    pub fn move_cam(&mut self, delta: glm::Vec3)
    {
        self.move_cam_lat(delta.x);
        self.move_cam_vert(delta.y);
        self.move_cam_long(delta.z);
    }

    pub fn move_cam_lat_locked(&mut self, delta: f32)
    {
        let delta_lat = self.orientation * (&self.world_right * delta);
        self.translation += glm::vec3(delta_lat.x, 0.0, delta_lat.z);
    }

    pub fn move_cam_vert_locked(&mut self, delta: f32)
    {
        self.move_world_vert(-delta);
    }

    pub fn move_cam_long_locked(&mut self, delta: f32)
    {
        let delta_lat = self.orientation * (&self.world_forward * delta);
        self.translation += glm::vec3(delta_lat.x, 0.0, delta_lat.z);
    }

    pub fn move_cam_locked(&mut self, delta: glm::Vec3)
    {
        self.move_cam_lat_locked(delta.x);
        self.move_cam_vert_locked(delta.y);
        self.move_cam_long_locked(delta.z);
    }

    /// Rotate Camera

    pub fn rotate_cam(&mut self, orientation: glm::Quat)
    {
        self.orientation = (&self.orientation * orientation).normalize();
    }

    pub fn rotate_cam_yaw(&mut self, theta: f32)
    {
        self.rotate_cam(glm::quat_angle_axis(glm::radians(&glm::vec1(theta)).x, &self.world_up));
        //self.rotate_cam(Quaternion::from_axis_angle(self.orientation * self.world_up, Deg(theta)));
    }

    pub fn rotate_cam_pitch(&mut self, theta: f32)
    {
        self.rotate_cam(glm::quat_angle_axis(glm::radians(&glm::vec1(theta)).x, &self.world_right));
        //self.rotate_cam(Quaternion::from_axis_angle(self.orientation * self.world_right, Deg(theta)));
    }

    pub fn rotate_cam_roll(&mut self, theta: f32)
    {
        self.rotate_cam(glm::quat_angle_axis(glm::radians(&glm::vec1(theta)).x, &self.world_forward));
        //self.rotate_cam(Quaternion::from_axis_angle(self.orientation * self.world_forward, Deg(theta)));
    }

    pub fn set_orientation(&mut self, orientation: glm::Quat)
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
    pub fn get_eye_pos(&self) -> glm::Vec3
    {
        self.eye_pos.clone()
    }
    
    pub fn get_looking_at(&self) -> glm::Vec3
    {
        self.looking_at.clone()
    }

    pub fn get_world_up(&self) -> glm::Vec3
    {
        self.world_up.clone()
    }

    pub fn get_world_forward(&self) -> glm::Vec3
    {
        self.world_forward.clone()
    }

    pub fn get_world_right(&self) -> glm::Vec3
    {
        self.world_right.clone()
    }

    pub fn get_world_translation(&self) -> glm::Vec3
    {
        self.translation.clone()
    }

    pub fn get_zoomed_eye_pos(&self) -> glm::Vec3
    {
        &self.eye_pos + (self.zoom * &self.world_forward)
    }

    /// Get the position of the camera's eye within the world
    pub fn get_world_eye_pos(&self) -> glm::Vec3
    {
        (&self.orientation * (self.get_zoomed_eye_pos() - (self.zoom * &self.world_forward * 2.0))) + &self.translation
    }

    /// Get the position of the camera within the world adjusted to be from
    /// the camera's view
    pub fn get_world_pos_adjusted(&self) -> glm::Vec3
    {
        -1.0 * self.get_world_eye_pos()
    }

    pub fn get_cam_pos(&self) -> glm::Vec3
    {
        -1.0 * &self.translation
    }

    pub fn get_orientation(&self) -> glm::Quat
    {
        self.orientation
    }

    pub fn get_zoom(&self) -> f32
    {
        self.zoom
    }

    pub fn view_matrix(&self) -> glm::Mat4
    {
        // let eye_pos = self.get_zoomed_eye_pos();
        /*let mut view = Matrix4::look_at(Point3::new(self.eye_pos.x, self.eye_pos.y, self.eye_pos.z), Point3::new(self.looking_at.x, self.looking_at.y, self.looking_at.z), self.world_up)
            * Matrix4::from(self.orientation);
        let mut
        view[3] = view[0] * self.translation[0] + view[1] * self.translation[1] + view[2] * self.translation[2] + view[3];
        view*/
        glm::identity()
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
    extern crate nalgebra_glm as glm;

    const TEST_CAM: Camera =
        Camera
        {
            eye_pos: glm::vec3(0.0, 0.0, 0.0),
            looking_at: glm::vec3(0.0, 0.0, 0.1),

            world_up: glm::vec3(0.0, 1.0, 0.0),
            world_forward: glm::vec3(0.0, 0.0, 0.1),
            world_right: glm::vec3(-0.1, 0.0, 0.0),

            orientation: glm::quat(1.0, 0.0, 0.0, 0.0),
            translation: glm::vec3(0.0, 0.0, 0.0),

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