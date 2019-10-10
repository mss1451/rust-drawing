extern crate winit;

use crate::Result;
use crate::color::*;
use crate::units::*;

use std::cell::Ref;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ColoredVertex {
	pub pos: [f32; 2],
	pub color: [f32; 4],
}

impl ColoredVertex {
	pub fn new(pos: [f32; 2], color: [f32; 4]) -> Self {
		ColoredVertex { pos, color }
	}
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TexturedVertex {
	pub pos: [f32; 2],
	pub tex_coords: [f32; 2],
	pub color: [f32; 4],
}

impl TexturedVertex {
	pub fn new(pos: [f32; 2], tex_coords: [f32; 2], color: [f32; 4]) -> Self {
		TexturedVertex {
			pos,
			tex_coords,
			color,
		}
	}
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TexturedY8Vertex {
	pub pos: [f32; 2],
	pub tex_coords: [f32; 2],
	pub color: [f32; 4],
}

impl TexturedY8Vertex {
	pub fn new(pos: [f32; 2], tex_coords: [f32; 2], color: [f32; 4]) -> Self {
		TexturedY8Vertex {
			pos,
			tex_coords,
			color,
		}
	}
}

pub trait Device {
	type Texture: Texture;
	type RenderTarget;
	type WindowTarget: WindowTarget;

	fn new() -> Result<Self>
	where
		Self: Sized;

	fn get_device_transform(size: PhysPixelSize) -> PhysPixelToDeviceTransform;

	fn create_window_target(
		&mut self,
		window_builder: winit::window::WindowBuilder,
		events_loop: &winit::event_loop::EventLoop<()>,
	) -> Result<Self::WindowTarget>;

	fn create_texture(
		&mut self,
		memory: Option<&[u8]>,
		width: u16,
		height: u16,
		format: ColorFormat,
		updatable: bool,
	) -> Result<Self::Texture>;

	fn create_render_target(
		&mut self,
		width: u16,
		height: u16,
	) -> Result<(Self::Texture, Self::RenderTarget)>;

	fn begin(&mut self, window_target: &Self::WindowTarget) -> Result<()>;

	fn clear(&mut self, target: &Self::RenderTarget, color: &Color);

	fn triangles_colored(
		&mut self,
		target: &Self::RenderTarget,
		vertices: &[ColoredVertex],
		transform: UnknownToDeviceTransform,
	);

	fn triangles_textured(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		vertices: &[TexturedVertex],
		transform: UnknownToDeviceTransform,
	);

	fn triangles_textured_y8(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		vertices: &[TexturedY8Vertex],
		transform: UnknownToDeviceTransform,
	);

	fn line(
		&mut self,
		target: &Self::RenderTarget,
		color: &Color,
		thickness: DeviceThickness,
		start_point: Point,
		end_point: Point,
		transform: UnknownToDeviceTransform,
	);

	fn rect_colored(
		&mut self,
		target: &Self::RenderTarget,
		color: &Color,
		rect: Rect,
		transform: UnknownToDeviceTransform,
	) {
		let p1 = [rect.origin.x, rect.origin.y];
		let p2 = [
			rect.origin.x + rect.size.width,
			rect.origin.y + rect.size.height,
		];

		self.triangles_colored(
			target,
			&[
				ColoredVertex::new([p1[0], p1[1]], *color),
				ColoredVertex::new([p2[0], p1[1]], *color),
				ColoredVertex::new([p1[0], p2[1]], *color),
				ColoredVertex::new([p2[0], p1[1]], *color),
				ColoredVertex::new([p2[0], p2[1]], *color),
				ColoredVertex::new([p1[0], p2[1]], *color),
			],
			transform,
		);
	}

	fn rect_textured(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		color: &Color,
		rect: Rect,
		transform: UnknownToDeviceTransform,
	) {
		self.rect_textured_sub(
			target,
			texture,
			filtering,
			color,
			rect,
			&[0.0, 0.0],
			&[1.0, 1.0],
			transform,
		)
	}

	fn rect_textured_sub(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		color: &Color,
		rect: Rect,
		uv1: &[f32; 2],
		uv2: &[f32; 2],
		transform: UnknownToDeviceTransform,
	) {
		let p1 = [rect.origin.x, rect.origin.y];
		let p2 = [
			rect.origin.x + rect.size.width,
			rect.origin.y + rect.size.height,
		];

		self.triangles_textured(
			target,
			texture,
			filtering,
			&[
				TexturedVertex::new([p1[0], p1[1]], [uv1[0], uv1[1]], *color),
				TexturedVertex::new([p2[0], p1[1]], [uv2[0], uv1[1]], *color),
				TexturedVertex::new([p1[0], p2[1]], [uv1[0], uv2[1]], *color),
				TexturedVertex::new([p2[0], p1[1]], [uv2[0], uv1[1]], *color),
				TexturedVertex::new([p2[0], p2[1]], [uv2[0], uv2[1]], *color),
				TexturedVertex::new([p1[0], p2[1]], [uv1[0], uv2[1]], *color),
			],
			transform,
		);
	}

	fn rect_textured_y8(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		color: &Color,
		rect: Rect,
		transform: UnknownToDeviceTransform,
	) {
		self.rect_textured_y8_sub(
			target,
			texture,
			filtering,
			color,
			rect,
			&[0.0, 0.0],
			&[1.0, 1.0],
			transform,
		)
	}

	fn rect_textured_y8_sub(
		&mut self,
		target: &Self::RenderTarget,
		texture: &Self::Texture,
		filtering: bool,
		color: &Color,
		rect: Rect,
		uv1: &[f32; 2],
		uv2: &[f32; 2],
		transform: UnknownToDeviceTransform,
	) {
		let p1 = [rect.origin.x, rect.origin.y];
		let p2 = [
			rect.origin.x + rect.size.width,
			rect.origin.y + rect.size.height,
		];

		self.triangles_textured_y8(
			target,
			texture,
			filtering,
			&[
				TexturedY8Vertex::new([p1[0], p1[1]], [uv1[0], uv1[1]], *color),
				TexturedY8Vertex::new([p2[0], p1[1]], [uv2[0], uv1[1]], *color),
				TexturedY8Vertex::new([p1[0], p2[1]], [uv1[0], uv2[1]], *color),
				TexturedY8Vertex::new([p2[0], p1[1]], [uv2[0], uv1[1]], *color),
				TexturedY8Vertex::new([p2[0], p2[1]], [uv2[0], uv2[1]], *color),
				TexturedY8Vertex::new([p1[0], p2[1]], [uv1[0], uv2[1]], *color),
			],
			transform,
		);
	}

	fn end(&mut self, window_target: &Self::WindowTarget);
}

pub trait WindowTarget: Sized {
	type RenderTarget;

	fn get_window(&self) -> Ref<winit::window::Window>;

	fn get_render_target(&self) -> &Self::RenderTarget;

	fn update_size(&mut self, width: u16, height: u16);

	fn swap_buffers(&mut self);
}

pub trait Texture: Sized {
	fn get_size(&self) -> (u16, u16);

	fn update(
		&mut self,
		memory: &[u8],
		offset_x: u16,
		offset_y: u16,
		width: u16,
		height: u16,
	) -> Result<()>;
}

///////////////////////////////////////////////////////////////////////
//
// backend specific extensions
//
///////////////////////////////////////////////////////////////////////

pub trait WindowTargetExt: Sized {
	type Context;

	fn get_context(&self) -> Ref<Self::Context>;
}
