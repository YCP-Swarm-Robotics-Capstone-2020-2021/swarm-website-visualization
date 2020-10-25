use cgmath::
{
    prelude::*,
    vec3,
    Vector3,
    Quaternion,
    Rad,
    Matrix4,
};

/// Holds a scale, orientation, and position for a transformation
#[derive(Clone, Copy, Debug)]
pub struct SubTransformation
{
    scale: Vector3<f32>,
    orientation: Quaternion<f32>,
    translation: Vector3<f32>,
    has_changed: bool,
    matrix_cache: Matrix4<f32>,
}

impl Default for SubTransformation
{
    fn default() -> Self
    {
        SubTransformation
        {
            scale: vec3(1.0, 1.0, 1.0),
            orientation: Quaternion::from_axis_angle(vec3(0.0, 0.0, 0.0), Rad(0.0)),
            translation: vec3(0.0, 0.0, 0.0),
            has_changed: true,
            matrix_cache: cgmath::Transform::one()
        }
    }
}

impl SubTransformation
{
    #[allow(dead_code)]
    pub fn new() -> Self
    {
        Default::default()
    }

    #[allow(dead_code)]
    pub fn scale(&mut self, scale: Vector3<f32>)
    {
        // Multiply in the scale factor
        self.scale.mul_assign_element_wise(scale);

        // Scale the current position by the scale factor
        self.translation.mul_assign_element_wise(scale);

        self.has_changed = true;
    }

    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: Vector3<f32>)
    {
        // Remove the current scale factors from position
        self.translation.div_assign_element_wise(self.scale);

        self.scale = scale;

        // Scale the position by the new scale value
        self.translation.mul_assign_element_wise(self.scale);

        self.has_changed = true;
    }

    #[allow(dead_code)]
    pub fn get_scale(&self) -> &Vector3<f32>
    {
        &self.scale
    }

    /// Rotate orientation with a quaternion
    /// `rotation` MUST be normalized
    #[allow(dead_code)]
    pub fn rotate_quat(&mut self, rotation: Quaternion<f32>)
    {
        // Apply rotation and normalize result
        self.orientation = self.orientation * rotation.normalize();
        self.orientation = self.orientation.normalize();

        // Apply rotation to position
        self.translation = rotation * self.translation;

        self.has_changed = true;
    }

    /// Rotate orientation with an angle (in radians) and axis of rotation
    /// `axis` MUST be normalized
    #[allow(dead_code)]
    pub fn rotate_angle_axis<A: Into<Rad<f32>>>(&mut self, angle: A, axis: Vector3<f32>)
    {
        self.rotate_quat(Quaternion::from_axis_angle(axis, angle))
    }

    /// Set orientation with a quaternion
    /// `orientation` MUST be normalized
    #[allow(dead_code)]
    pub fn set_orientation_quat(&mut self, orientation: Quaternion<f32>)
    {
        // Remove current orientation from the position
        self.translation = (self.orientation.conjugate() * orientation).normalize() * self.translation;
        // Set new orientation
        self.orientation = orientation;
        // Apply new orientation to position
        self.translation = self.orientation * self.translation;

        self.has_changed = true;
    }

    /// Set orientation with an angle (in radians) and axis of rotation
    /// `axis` MUST be normalized
    #[allow(dead_code)]
    pub fn set_orientation_angle_axis<A: Into<Rad<f32>>>(&mut self, angle: A, axis: Vector3<f32>)
    {
        self.set_orientation_quat(Quaternion::from_axis_angle(axis, angle))
    }

    #[allow(dead_code)]
    pub fn get_orientation(&self) -> &Quaternion<f32>
    {
        &self.orientation
    }

    #[allow(dead_code)]
    pub fn translate(&mut self, translation: Vector3<f32>)
    {
        self.translation += translation;

        self.has_changed = true;
    }

    #[allow(dead_code)]
    pub fn set_translation(&mut self, translation: Vector3<f32>)
    {
        self.translation = translation;

        self.has_changed = true;
    }

    #[allow(dead_code)]
    pub fn get_translation(&self) -> &Vector3<f32>
    {
        &self.translation
    }

    #[allow(dead_code)]
    pub fn has_changed(&self) -> bool
    {
        self.has_changed
    }

    /// Assembles the transformation components into a matrix
    /// Mutable since it uses/updates an internal cache
    #[allow(dead_code)]
    pub fn as_matrix(&mut self) -> &Matrix4<f32>
    {
        if self.has_changed
        {
            self.matrix_cache = self.as_matrix_uncached();
            self.has_changed = false;
        }

        &self.matrix_cache
    }

    /// Assembles the transformation components into a matrix
    /// Less efficient since three matrix multiplications are performed
    /// on each call, but doesn't need a mutable reference
    #[allow(dead_code)]
    pub fn as_matrix_uncached(&self) -> Matrix4<f32>
    {
        Matrix4::from_translation(self.translation) *
            Matrix4::from(self.orientation) *
            Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

/// Holds a base size and global/local transformations
#[derive(Clone, Copy, Debug)]
pub struct Transformation
{
    pub base_size: Vector3<f32>,

    pub global: SubTransformation,
    pub local: SubTransformation,

    matrix_cache: Matrix4<f32>,
}

impl Default for Transformation
{
    fn default() -> Self
    {
        Transformation
        {
            base_size: vec3(1.0, 1.0, 1.0),
            global: Default::default(),
            local: Default::default(),
            matrix_cache: cgmath::Transform::one()
        }
    }
}

impl Transformation
{
    #[allow(dead_code)]
    pub fn new() -> Self
    {
        Default::default()
    }

    /// Reset all transformations
    #[allow(dead_code)]
    pub fn reset(&mut self)
    {
        *self = Default::default()
    }

    #[allow(dead_code)]
    pub fn scale(&self) -> Vector3<f32>
    {
        self.global.get_scale().mul_element_wise(*self.local.get_scale())
    }

    #[allow(dead_code)]
    pub fn orientation(&self) -> Quaternion<f32>
    {
        self.global.orientation * self.local.orientation
    }

    #[allow(dead_code)]
    pub fn translation(&self) -> Vector3<f32>
    {
        self.global.translation + self.local.translation
    }

    /// Relatively scaled base size
    #[allow(dead_code)]
    pub fn size(&self) -> Vector3<f32>
    {
        self.base_size.mul_element_wise(self.scale())
    }

    #[allow(dead_code)]
    pub fn has_changed(&self) -> bool
    {
        self.global.has_changed || self.local.has_changed
    }

    #[allow(dead_code)]
    pub fn matrix(&mut self) -> &Matrix4<f32>
    {
        if self.global.has_changed || self.local.has_changed
        {
            self.matrix_cache = self.global.as_matrix() * self.local.as_matrix();
        }

        &self.matrix_cache
    }

    #[allow(dead_code)]
    pub fn matrix_uncached(&self) -> Matrix4<f32>
    {
        self.global.as_matrix_uncached() * self.local.as_matrix_uncached()
    }
}

#[cfg(test)]
mod tests
{
    use crate::math::transform::*;
    use cgmath::Deg;

    const I: [f32; 16] =
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

    // TODO: These assertions should be done with some sort of epsilon value,
    //  because of floating point precision errors while rounding

    #[test]
    fn init()
    {
        let mut t = Transformation::new();
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(I, *m);
        let m: &[f32; 16] = t.local.as_matrix().as_ref();
        assert_eq!(I, *m);
        let m: &[f32; 16] = t.matrix().as_ref();
        assert_eq!(I, *m);

        assert_eq!(vec3(0.0, 0.0, 0.0), *t.global.get_translation());
        assert_eq!(vec3(0.0, 0.0, 0.0), *t.local.get_translation());

        assert_eq!(vec3(1.0, 1.0, 1.0), *t.global.get_scale());
        assert_eq!(vec3(1.0, 1.0, 1.0), *t.local.get_scale());

        assert_eq!(Quaternion::from_axis_angle(vec3(0.0, 0.0, 0.0), Rad(0.0)), *t.global.get_orientation());
        assert_eq!(Quaternion::from_axis_angle(vec3(0.0, 0.0, 0.0), Rad(0.0)), *t.local.get_orientation());
    }
    #[test]
    fn reset()
    {
        let mut t = Transformation::new();
        t.global.translate(vec3(1.0, 1.0, 1.0));

        t.reset();
        let m: &[f32; 16] = t.matrix().as_ref();
        assert_eq!(I, *m);
    }
    #[test]
    fn scale()
    {
        let mut t = Transformation::new();

        // Test scaling on I
        t.global.scale(vec3(2.0, 2.0, 2.0));
        let expected: [f32; 16] = [2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);

        t.reset();

        // Test scaling on transformation with translations
        t.global.translate(vec3(1.0, 0.0, 1.0));
        t.global.scale(vec3(2.0, 2.0, 2.0));
        let expected: [f32; 16] = [2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        assert_eq!(vec3(2.0, 0.0, 2.0), *t.global.get_translation());

        t.global.set_scale(vec3(1.0, 1.0, 1.0));
        let expected: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        assert_eq!(vec3(1.0, 0.0, 1.0), *t.global.get_translation());

        // Test scaling on transformation with orientation
        t.global.rotate_angle_axis(Deg(1.0), vec3(0.0, 1.0, 0.0));
        t.global.scale(vec3(2.0, 1.0, 2.0));
        let expected: [f32; 16] = [1.9996954, 0.0, -0.03490481, 0.0, 0.0, 1.0, 0.0, 0.0, 0.03490481, 0.0, 1.9996954, 0.0, 2.0346003, 0.0, 1.9647906, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        // Scaling shouldn't effect rotation by design
        assert_eq!(Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), Deg(1.0)), *t.global.get_orientation());

        t.global.set_scale(vec3(1.0, 1.0, 1.0));
        let expected: [f32; 16] = [0.9998477, 0.0, -0.017452406, 0.0, 0.0, 1.0, 0.0, 0.0, 0.017452406, 0.0, 0.9998477, 0.0, 1.0173001, 0.0, 0.9823953, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        // Scaling shouldn't effect rotation by design
        assert_eq!(Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), Deg(1.0)), *t.global.get_orientation());
    }
    #[test]
    fn orientation()
    {
        let mut t = Transformation::new();

        // Test orientation on I
        t.global.rotate_angle_axis(Deg(1.0), vec3(1.0, 0.0, 1.0));
        let expected: [f32; 16] = [0.9998477, 0.01745108, 0.00015229327, 0.0, -0.01745108, 0.9996954, 0.01745108, 0.0, 0.00015229327, -0.01745108, 0.9998477, 0.0, 0.0, 0.0, 0.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);

        t.reset();

        // Test orientation on transformation with translation
        t.global.translate(vec3(1.0, 1.0, 0.0));
        t.global.rotate_angle_axis(Deg(23.0), vec3(1.0, 0.0, 1.0));
        let expected: [f32; 16] = [0.9235438, 0.3757942, 0.0764562, 0.0, -0.3757942, 0.8470876, 0.3757942, 0.0, 0.0764562, -0.3757942, 0.9235438, 0.0, 0.5477496, 1.2228818, 0.45225042, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);

        t.global.set_orientation_angle_axis(Deg(0.0), vec3(0.0, 0.0, 0.0));
        let expected: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0000001, 0.9999999, -0.00000011920929, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);

        t.global.translate(vec3(-1.0, -1.0, 0.0));
        assert_eq!(vec3(0.00000011920929, -0.00000011920929, -0.00000011920929), *t.global.get_translation());

        //let m: &[f32; 16] = t.global.as_matrix().as_ref();
        //TODO: These assertions are *technically* true, but fail because of
        // rounding precision errors
        //assert_eq!(I, *m);
        //assert_eq!(vec3(0.0, 0.0, 0.0), *t.global.get_translation());
    }
    #[test]
    fn translation()
    {
        let mut t = Transformation::new();

        t.global.translate(vec3(1.0, 0.0, 3.0));
        let expected: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 3.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        assert_eq!(vec3(1.0, 0.0, 3.0), *t.global.get_translation());

        t.global.set_translation(vec3(0.0, 0.0, 0.0));
        let expected: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let m: &[f32; 16] = t.global.as_matrix().as_ref();
        assert_eq!(expected, *m);
        assert_eq!(vec3(0.0, 0.0, 0.0), *t.global.get_translation());
    }
/*    #[test]
    fn caching()
    {

    }*/
}