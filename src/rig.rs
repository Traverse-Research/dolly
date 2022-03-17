use crate::{
    driver::{RigDriver, RigDriverTraits},
    handedness::{Handedness, RightHanded},
    transform::Transform,
};
use core::fmt::Debug;
use std::marker::PhantomData;

/// A chain of drivers, calculating displacements, and animating in succession.
#[derive(Debug)]
pub struct CameraRig<H: Handedness + 'static = RightHanded> {
    ///
    pub drivers: Vec<Box<dyn RigDriverTraits<H>>>,

    ///
    pub final_transform: Transform<H>,

    ty: PhantomData<H>,
}

// Prevents user calls to `RigDriver::update`. All updates must come from `CameraRig::update`.
struct RigUpdateToken;

///
pub struct RigUpdateParams<'a, H: Handedness + 'static> {
    ///
    pub parent: &'a Transform<H>,
    ///
    pub delta_time_seconds: f32,

    ty: PhantomData<H>,

    _token: RigUpdateToken,
}

impl<H: Handedness + 'static> CameraRig<H> {
    /// Returns the first driver of the matching type. Panics if no such driver is present.
    pub fn driver_mut<T: RigDriver<H> + 'static>(&mut self) -> &mut T {
        self.try_driver_mut::<T>().unwrap_or_else(|| {
            panic!(
                "No {} driver found in the CameraRig",
                std::any::type_name::<T>()
            )
        })
    }

    /// Returns the Some with the first driver of the matching type, or `None` if no such driver is present.
    pub fn try_driver_mut<T: RigDriver<H> + 'static>(&mut self) -> Option<&mut T> {
        self.drivers
            .iter_mut()
            .find_map(|driver| driver.as_mut().as_any_mut().downcast_mut::<T>())
    }

    /// Runs all the drivers in sequence, animating the rig, and producing a final transform of the camera.
    ///
    /// Camera rigs are approximately framerate independent, so `update` can be called at any frequency.
    pub fn update(&mut self, delta_time_seconds: f32) -> Transform<H> {
        let mut parent_transform = Transform::IDENTITY;

        for driver in self.drivers.iter_mut() {
            let transform = driver.update(RigUpdateParams {
                parent: &parent_transform,
                delta_time_seconds,
                ty: PhantomData,
                _token: RigUpdateToken,
            });

            parent_transform = transform;
        }

        self.final_transform = parent_transform;
        self.final_transform
    }

    /// Use this to make a new rig
    pub fn builder() -> CameraRigBuilder<H> {
        CameraRigBuilder {
            drivers: Default::default(),
            ty: PhantomData,
        }
    }
}

///
pub struct CameraRigBuilder<H: Handedness + 'static> {
    drivers: Vec<Box<dyn RigDriverTraits<H>>>,
    ty: PhantomData<H>,
}

impl<H: Handedness + 'static> CameraRigBuilder<H> {
    ///
    pub fn with(mut self, driver: impl RigDriverTraits<H>) -> Self {
        self.drivers.push(Box::new(driver));
        self
    }

    ///
    pub fn build(self) -> CameraRig<H> {
        let mut rig = CameraRig {
            drivers: self.drivers,
            // Initialize with a dummy identity transform. Will be overridden in a moment.
            final_transform: Transform::IDENTITY,
            ty: PhantomData,
        };

        // Update once to find the final transform
        rig.update(0.0);
        rig
    }
}

//

// pub trait BuilderWithHandedness<H: Handedness + 'static> {
//     fn build(self) -> CameraRig<H>;
// }

// impl BuilderWithHandedness<RightHanded> for CameraRigBuilder<RightHanded> {
//     fn build(self) -> CameraRig<RightHanded> {
//         self.build_internal()
//     }
// }

// impl BuilderWithHandedness<LeftHanded> for CameraRigBuilder<LeftHanded> {
//     fn build(self) -> CameraRig<LeftHanded> {
//         self.build_internal()
//     }
// }

// EHRM.............
// fn test() {
//     type CameraRig2 = CameraRig<LeftHanded>;

//     let cam = CameraRig2::builder().build();
// }
