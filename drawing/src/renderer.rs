use crate::backend::ColoredVertex;
use crate::backend::Device;
use crate::backend::RenderTarget;
use crate::backend::Scissor;
use crate::font::Font;
use crate::font::FontParams;
use crate::paint::Paint;
use crate::primitive::*;
use crate::resources::*;
use crate::units::*;
use crate::utils::path::FlattenedPath;
use crate::Result;

use std::convert::Into;

pub struct Renderer;

impl Renderer {
	pub fn new() -> Self {
		Renderer {}
	}

	pub fn draw<D: Device, F: Font<D>>(
		&mut self,
		device: &mut D,
		render_target: &D::RenderTarget,
		primitives: &Vec<Primitive>,
		resources: &mut Resources<D, F>,
	) -> Result<()> {
		let physical_pixel_to_device_transform = render_target.get_device_transform();
		let user_pixel_to_physical_pixel_transform = UserPixelToPhysPixelTransform::identity();
		let user_pixel_to_device_transform = user_pixel_to_physical_pixel_transform
			.post_transform(&physical_pixel_to_device_transform);
		let unknown_to_device_transform = UnknownToDeviceTransform::from_row_major_array(
			user_pixel_to_device_transform.to_row_major_array(),
		);

		for primitive in primitives {
			match primitive {
				&Primitive::Line {
					ref color,
					thickness,
					start_point,
					end_point,
				} => {
					let thickness = user_pixel_to_device_transform
						.transform_point(UserPixelPoint::new(thickness.get(), thickness.get()))
						.x;

					device.line(
						&render_target,
						color,
						DeviceThickness::new(thickness),
						start_point.to_untyped(),
						end_point.to_untyped(),
						unknown_to_device_transform,
					);
				}

				&Primitive::Rectangle { ref color, rect } => device.rect_colored(
					&render_target,
					color,
					rect.to_untyped(),
					unknown_to_device_transform,
				),

				&Primitive::Image {
					resource_key,
					rect,
					uv,
				} => {
					if let Some(texture) = resources.textures_mut().get(&resource_key) {
						device.rect_textured(
							&render_target,
							&texture,
							false,
							&[1.0f32, 1.0f32, 1.0f32, 1.0f32],
							rect.to_untyped(),
							&uv,
							unknown_to_device_transform,
						);
					}
				}

				&Primitive::Text {
					ref resource_key,
					ref color,
					position,
					clipping_rect,
					size,
					ref text,
				} => {
					if let Some(font) = resources.fonts_mut().get_mut(&resource_key.to_string()) {
						font.draw(
							device,
							&render_target,
							color,
							text,
							position.to_untyped(),
							clipping_rect.to_untyped(),
							FontParams { size: size as u8 },
							unknown_to_device_transform,
						)?;
					}
				}

				&Primitive::Stroke {
					ref path,
					ref thickness,
					ref brush,
				} => {
					let scale = 1.0f32; // TODO: take from transform? xform.average_scale()?
					let stroke_width = thickness * scale; //.clamped(0.0, 200.0);
					let aspect_ratio = render_target.get_aspect_ratio();
					let flattened_path = Self::get_stroke_path(
						path,
						stroke_width,
						&Default::default(),
						aspect_ratio,
					);
					let paint = Paint::from_brush(brush);

					device.stroke(
						&render_target,
						&paint,
						&flattened_path.paths,
						stroke_width,
						1.0f32 / aspect_ratio,
						Scissor {
							xform: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
							extent: [-1.0, -1.0],
						},
						CompositeOperation::Basic(BasicCompositeOperation::SrcOver).into(),
						unknown_to_device_transform,
					);
				}

				&Primitive::StrokeStyled {
					ref path,
					ref thickness,
					ref brush,
					ref style,
				} => {
					let scale = 1.0f32; // TODO: take from transform? xform.average_scale()?
					let stroke_width = thickness * scale; //.clamped(0.0, 200.0);
					let aspect_ratio = render_target.get_aspect_ratio();
					let flattened_path =
						Self::get_stroke_path(path, stroke_width, style, aspect_ratio);
					let paint = Paint::from_brush(brush);

					device.stroke(
						&render_target,
						&paint,
						&flattened_path.paths,
						stroke_width,
						1.0f32 / aspect_ratio,
						Scissor {
							xform: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
							extent: [-1.0, -1.0],
						},
						CompositeOperation::Basic(BasicCompositeOperation::SrcOver).into(),
						unknown_to_device_transform,
					);
				}

				&Primitive::Fill {
					ref path,
					ref brush,
				} => {
					let aspect_ratio = render_target.get_aspect_ratio();
					let flattened_path = Self::get_fill_path(path, aspect_ratio);
					let paint = Paint::from_brush(brush);

					device.fill(
						&render_target,
						&paint,
						&flattened_path.paths,
						flattened_path.bounds,
						1.0f32 / aspect_ratio,
						Scissor {
							xform: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
							extent: [-1.0, -1.0],
						},
						CompositeOperation::Basic(BasicCompositeOperation::SrcOver).into(),
						unknown_to_device_transform,
					);

					/*for path in flattened_path.paths {
						let fill_vertices = path.get_fill();
						if !fill_vertices.is_empty() {}

						let stroke_vertices = path.get_stroke();
						if !stroke_vertices.is_empty() {}

						let color = [1.0f32, 1.0f32, 0.0f32, 0.5f32];
						let mut arr: [ColoredVertex; 3] = [
							ColoredVertex {
								pos: fill_vertices[0].pos,
								color: color,
							},
							ColoredVertex {
								pos: fill_vertices[0].pos,
								color: color,
							},
							ColoredVertex {
								pos: fill_vertices[0].pos,
								color: color,
							},
						];

						for i in 2..fill_vertices.len() {
							arr[1].pos = fill_vertices[i - 1].pos;
							arr[2].pos = fill_vertices[i].pos;

							device.triangles_colored(
								render_target,
								&arr,
								unknown_to_device_transform,
							);
						}
					}*/
				}

				&Primitive::ClipRect {
					ref rect,
					ref primitives,
				} => {
					device.save_state();

					device.set_clip_rect(*rect);
					self.draw(device, render_target, primitives, resources)?;

					device.restore_state();
				}

				&Primitive::ClipPath {
					ref path,
					ref primitives,
				} => {
					device.save_state();

					device.set_clip_path(path);
					self.draw(device, render_target, primitives, resources)?;

					device.restore_state();
				}

				&Primitive::Transform {
					ref transform,
					ref primitives,
				} => {
					device.save_state();

					device.transform(*transform);
					self.draw(device, render_target, primitives, resources)?;

					device.restore_state();
				}

				&Primitive::Composite {
					ref color,
					ref primitives,
				} => {
					let size = render_target.get_size();
					let (texture2, texture2_view) =
						device.create_render_target(size.0 as u16, size.1 as u16)?;

					device.clear(&texture2_view, &[0.0f32, 0.0f32, 0.0f32, 0.0f32]);

					self.draw(device, &texture2_view, &primitives, resources)?;

					device.rect_textured(
						render_target,
						&texture2,
						false,
						&color,
						Rect::new(
							Point::new(0.0f32, 0.0f32),
							Size::new(size.0 as f32, size.1 as f32),
						),
						&[0.0, 0.0, 1.0, 1.0],
						unknown_to_device_transform,
					);
				}
			}
		}

		Ok(())
	}

	fn get_stroke_path(
		path: &Vec<PathElement>,
		stroke_width: f32,
		stroke_style: &StrokeStyle,
		aspect_ratio: f32,
	) -> FlattenedPath {
		let mut flattened_path =
			FlattenedPath::new(path, 0.01f32 / aspect_ratio, 0.25f32 / aspect_ratio);
		let anitialias = true;
		let fringe_width = 1.0f32 / aspect_ratio;
		if anitialias {
			flattened_path.expand_stroke(
				stroke_width * 0.5f32,
				fringe_width,
				stroke_style.line_cap,
				stroke_style.line_join,
				stroke_style.miter_limit,
				0.25f32 / aspect_ratio,
			);
		} else {
			flattened_path.expand_stroke(
				stroke_width * 0.5f32,
				0.0,
				stroke_style.line_cap,
				stroke_style.line_join,
				stroke_style.miter_limit,
				0.25f32 / aspect_ratio,
			);
		}
		flattened_path
	}

	fn get_fill_path(path: &Vec<PathElement>, aspect_ratio: f32) -> FlattenedPath {
		let mut flattened_path =
			FlattenedPath::new(path, 0.01f32 / aspect_ratio, 0.25f32 / aspect_ratio);
		let anitialias = true;
		let fringe_width = 1.0f32 / aspect_ratio;
		if anitialias {
			flattened_path.expand_fill(fringe_width, LineJoin::Miter, 2.4f32, fringe_width);
		} else {
			flattened_path.expand_fill(0.0f32, LineJoin::Miter, 2.4f32, fringe_width);
		}
		flattened_path
	}
}
