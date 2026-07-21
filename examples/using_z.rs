// use the Z callback to flag intersections by setting z = 1;

use clipper2_rust::utils::svg::{
    svg_add_clip_64, svg_add_clip_d, svg_add_solution_64, svg_add_solution_d, svg_add_subject_64,
    svg_add_subject_d, svg_save_to_file, SvgReader, SvgWriter,
};
use clipper2_rust::{
    distance, ellipse_rect64, ellipse_rect_d, get_bounds_paths, make_path64, make_path_d, ClipType,
    Clipper64, ClipperD, FillRule, Paths64, PathsD, Rect64, RectD,
};

fn main() {
    test1_64();
    test1_double();
    test2_double();
}

fn byte_to_rainbow_color(b: u8) -> u32 {
    let b2: u8;
    match b / 43 {
        //0..42
        0 => {
            b2 = (b - 0) * 6;
            return 0xFFFF0000u32 | ((b2 as u32) << 8); // 0xFFFF0000 -> 0xFFFFFF00 (red..yellow)
        }
        //43..85
        1 => {
            b2 = (85 - b) * 6;
            return 0xFF00FF00 | ((b2 as u32) << 16); // 0xFFFFFF00 -> 0xFF00FF00 (yellow..lime)
        }
        //86..128
        2 => {
            b2 = (b - 86) * 6;
            return 0xFF00FF00 | (b2 as u32); // 0xFF00FF00 -> 0xFF00FFFF (lime..aqua)
        }
        //129..171
        3 => {
            b2 = (171 - b) * 6;
            return 0xFF0000FF | ((b2 as u32) << 8); // 0xFF00FFFF -> 0xFF0000FF (aqua..blue)
        }
        //172..214
        4 => {
            b2 = (b - 172) * 6;
            return 0xFF0000FF | ((b2 as u32) << 16); // 0xFF0000FF -> 0xFFFF00FF (blue..fuschia)
        }
        //215..255
        _ => {
            b2 = (255 - b) * 6;
            return 0xFFFF0000 | (b2 as u32); // 0xFF0000FF -> 0xFFFF00FF (fuschia..red)
        }
    }
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

fn display_as_svg_d(
    filename: &str,
    subject: &PathsD,
    clip: &PathsD,
    solution: &PathsD,
    multi_color: bool,
) {
    let mut svg = SvgWriter::new(0);
    if !subject.is_empty() {
        svg_add_subject_d(&mut svg, subject, FillRule::NonZero);
    }
    if !clip.is_empty() {
        svg_add_clip_d(&mut svg, clip, FillRule::NonZero);
    }
    if !solution.is_empty() {
        if multi_color {
            #[cfg(feature = "using_z")]
            {
                for path in solution.iter() {
                    // set color using the average 'z' for each triangle
                    let d: u8 = if path.len() == 3 {
                        ((path[0].z + path[1].z + path[2].z) as f64 / 3.0) as u8
                    } else {
                        128
                    };
                    svg.add_path_d(
                        path,
                        false,
                        FillRule::NonZero,
                        byte_to_rainbow_color(d),
                        0x80808080,
                        0.8,
                        false,
                    );
                }
            }
            #[cfg(not(feature = "using_z"))]
            {
                // just set a random color
                // SvgAddRCSolution(svg, *solution, FillRule::NonZero, false);
            }
        } else {
            svg_add_solution_d(&mut svg, solution, FillRule::NonZero, false);
        }
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
    display_as_svg_d("TestingZ1_D.svg", &subject, &ellipses, &solution, false);
}

fn test2_double() {
    let mut sr = SvgReader::new();
    sr.load_from_file(".\\TriSamples\\coral3.svg");
    let mut subject = sr.get_paths();
    let mut sol = PathsD::new();
    let r = get_bounds_paths(&subject);
    let mp = r.mid_point();
    let d = (mp.y - r.top) / 255.0;
    // for each point in subject, set its 'z' as a
    // relative distance fron 'mp' (scaled to 255)
    for path in subject.iter_mut() {
        for pt in path.iter_mut() {
            pt.z = (distance(pt.clone(), mp) / d) as i64;
        }
    }
    // TODO: we need to port Triangulate
    // Triangulate(subject, 0, sol, true);
    display_as_svg_d("coral3_t2.svg", &vec![], &vec![], &sol, true);
}
