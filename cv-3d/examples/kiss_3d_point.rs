use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra::Point3;

fn main() {
    let mut window = Window::new("kiss3d: points");
    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0);
    window.set_background_color(0.9, 0.9, 0.9);
    while window.render() {
        let a = Point3::new(-0.1, -0.1, 0.0);
        let b = Point3::new(0.0, 0.1, 0.0);
        let c = Point3::new(0.1, -0.1, 0.0);
        window.draw_point(&a, &Point3::new(1.0, 0.0, 0.0));
        window.draw_point(&b, &Point3::new(0.0, 1.0, 0.0));
        window.draw_point(&c, &Point3::new(0.0, 0.0, 1.0));
    }
}
