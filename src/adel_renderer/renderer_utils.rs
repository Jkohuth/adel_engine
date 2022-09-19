use std::sync::Arc;
#[allow(unused_imports)]
use glam::{Mat2, Mat4};
use vulkano::{
        buffer::{BufferUsage, CpuAccessibleBuffer},
        device::Device,
        memory::DeviceMemoryAllocationError,
};

use crate::adel_renderer::{
    TransformComponent,
    Transform2dComponent,
    Vertex,
    Vertex2d,
    vs_push::ty::PushConstantData,
    vs_2d_push::ty::PushConstantData2d,
};

pub fn create_vertex_buffers(device: &Arc<Device>, verticies: Vec<Vertex>) -> Result<Arc<CpuAccessibleBuffer<[Vertex]>>, DeviceMemoryAllocationError> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            verticies)
}
pub fn create_vertex_buffers_2d(device: &Arc<Device>, verticies: Vec<Vertex2d>) -> Result<Arc<CpuAccessibleBuffer<[Vertex2d]>>, DeviceMemoryAllocationError> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            verticies)
}

pub fn create_index_buffers(device: &Arc<Device>, indicies: Vec<u32>) -> Result<Arc<CpuAccessibleBuffer<[u32]>>, DeviceMemoryAllocationError> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            indicies)
}


pub fn create_push_constant_data_2d(transform: &Transform2dComponent) -> PushConstantData2d {
    PushConstantData2d {
        transform: transform.mat2().to_cols_array_2d(),
        offset: transform.translation.into(),
        color: [0.0, 0.0, 0.0],
        _dummy0: [0,0,0,0,0,0,0,0],
    }
}

pub fn create_push_constant_data(camera_projection: Mat4, transform: &TransformComponent) -> PushConstantData {
    PushConstantData {
        transform: (camera_projection * transform.mat4_less_computation()).to_cols_array_2d(),
        color: [0.0, 0.0, 0.0],
    }
}