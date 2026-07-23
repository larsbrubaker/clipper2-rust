// use the Z callback to flag intersections by setting z = 1;

use clipper2_rust::utils::svg::{
    svg_add_clip_64, svg_add_clip_d, svg_add_solution_64, svg_add_solution_d, svg_add_subject_64,
    svg_add_subject_d, svg_save_to_file, SvgWriter,
};
use clipper2_rust::{
    ellipse_rect64, ellipse_rect_d, make_path64, make_path_d, ClipType, Clipper64, ClipperD,
    FillRule, Paths64, PathsD, Rect64, RectD,
};

fn main() {
    test1_64();
    test1_double();
}

fn display_as_svg64(filename: &str, subject: &Paths64, clip: &Paths64, solution: &Paths64) {
    let mut svg = SvgWriter::new(0);
    if !subject.is_empty() {
        svg_add_subject_64(&mut svg, subject, FillRule::NonZero);
    }
    if !clip.is_empty() {
        svg_add_clip_64(&mut svg, clip, FillRule::NonZero);
    }
    if !solution.is_empty() {
        svg_add_solution_64(&mut svg, solution, FillRule::NonZero, false);
    }
    svg_save_to_file(&mut svg, filename, 320, 320, 0);
}

fn display_as_svg_d(filename: &str, subject: &PathsD, clip: &PathsD, solution: &PathsD) {
    let mut svg = SvgWriter::new(0);
    if !subject.is_empty() {
        svg_add_subject_d(&mut svg, subject, FillRule::NonZero);
    }
    if !clip.is_empty() {
        svg_add_clip_d(&mut svg, clip, FillRule::NonZero);
    }
    if !solution.is_empty() {
        svg_add_solution_d(&mut svg, solution, FillRule::NonZero, false);
    }
    svg_save_to_file(&mut svg, filename, 320, 320, 0);
}

fn test1_64() {
    let mut subject = Paths64::new();
    let mut solution = Paths64::new();
    let mut c64 = Clipper64::new();

    subject.push(make_path64(&[100, 50, 10, 79, 65, 2, 65, 98, 10, 21]));
    c64.add_subject(&subject);
    c64.set_z_callback(|_, _, _, _, pt| {
        pt.z = 1;
    });
    c64.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);

    let mut ellipses = Paths64::new();
    if solution.len() > 0 {
        // draw circles around intersection points - flagged by z == 1
        let r = 3.0;
        for pt in solution[0].iter() {
            if pt.z == 1 {
                ellipses.push(ellipse_rect64(
                    &Rect64::new(
                        pt.x - r as i64,
                        pt.y - r as i64,
                        pt.x + r as i64,
                        pt.y + r as i64,
                    ),
                    11,
                ));
            }
        }
    }
    display_as_svg64("TestingZ1_64.svg", &subject, &ellipses, &solution);
}

fn test1_double() {
    let mut subject = PathsD::new();
    let mut solution = PathsD::new();
    let mut c = ClipperD::new(2);

    subject.push(make_path_d(&[
        100.0, 50.0, 10.0, 79.0, 65.0, 2.0, 65.0, 98.0, 10.0, 21.0,
    ]));
    c.add_subject(&subject);
    c.set_z_callback(|_, _, _, _, pt| {
        pt.z = 1;
    });
    c.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);

    let mut ellipses = PathsD::new();
    if solution.len() > 0 {
        // draw circles around intersection points
        let r = 3.0;
        for pt in solution[0].iter() {
            if pt.z == 1 {
                ellipses.push(ellipse_rect_d(
                    &RectD::new(pt.x - r, pt.y - r, pt.x + r, pt.y + r),
                    11,
                ));
            }
        }
    }
    display_as_svg_d("TestingZ1_D.svg", &subject, &ellipses, &solution);
}
