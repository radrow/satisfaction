use itertools::Itertools;
use std::{collections::HashMap, error::Error, path::Path, time::Duration};

use plotters::prelude::*;

pub fn plot_runtimes(
    measurement: HashMap<String, Vec<Duration>>,
    path: impl AsRef<Path>,
    size: (u32, u32),
) -> Result<(), Box<dyn Error>> {
    let drawing_area = SVGBackend::new(path.as_ref(), size).into_drawing_area();
    drawing_area.fill(&WHITE)?;

    let max_instances = measurement
        .values()
        .map(|vec| vec.len())
        .max()
        .expect("Measurement was empty!");

    let max_duration = measurement
        .values()
        .filter_map(|vec| vec.iter().max())
        .max()
        .expect("Measurement was empty!");

    let max_duration = max_duration.as_millis();
    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(30)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(0..max_instances, 0..max_duration)?;

    chart
        .configure_mesh()
        .x_desc("Number of solved instances")
        .y_desc("CPU-Time")
        .draw()?;

    let mut colors = vec![
        (255, 0, 0),
        (0, 255, 0),
        (0, 255, 255),
        (0, 0, 255),
        (255, 0, 255),
    ]
    .into_iter()
    .cycle();
    for (name, times) in measurement.iter() {
        //let line_color = HSLColor(color, 0.7, 0.5);
        let (r, g, b) = colors.next().unwrap();
        let line_color = RGBColor(r, g, b);
        let point_color = RGBColor(r, g, b);

        let y = times
            .iter()
            .map(|dur| dur.as_millis())
            .sorted()
            .collect::<Vec<_>>();

        let points = PointSeries::of_element(
            y.iter().cloned().enumerate(),
            5,
            &point_color,
            &|c, s, st| Circle::new(c, s, st),
        );
        chart.draw_series(points)?;

        let lines = LineSeries::new(y.into_iter().enumerate(), &line_color);
        chart
            .draw_series(lines)?
            .label(name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x - 20, y)], &line_color));
    }
    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .margin(5)
        .draw()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::plot_runtimes;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn plotting() {
        let mut map = HashMap::new();
        map.insert(
            "1".to_string(),
            vec![10, 5, 7, 9, 200, 3]
                .into_iter()
                .map(|i| Duration::from_secs(i))
                .collect(),
        );
        map.insert(
            "2".to_string(),
            vec![1, 300, 240, 7, 50, 200, 3]
                .into_iter()
                .map(|i| Duration::from_secs(i))
                .collect(),
        );

        plot_runtimes(map, "test.svg", (1280, 720)).unwrap();
    }
}
