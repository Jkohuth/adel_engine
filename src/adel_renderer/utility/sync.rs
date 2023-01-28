use ash::vk;
use anyhow::Result;

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

impl SyncObjects {
    pub fn new(
        device: &ash::Device,
        max_frame_in_flight: usize
    ) -> Result<Self> {
        let mut sync_objects = SyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            inflight_fences: vec![],
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder()
            .build();

        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        for _ in 0..max_frame_in_flight {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)?;
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)?;
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)?;

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.inflight_fences.push(inflight_fence);
            }
        }

        Ok(sync_objects)
    }

    pub unsafe fn cleanup_sync_objects(&mut self, device: &ash::Device, max_frames_in_flight: usize) {
            for i in 0..max_frames_in_flight {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finished_semaphores[i], None);
                device.destroy_fence(self.inflight_fences[i], None);
            }

    }
}
