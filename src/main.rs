use ff::PrimeField;

use gmimc_rust_test::gmimc::gmimc_erf;

use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "./results/sample_drawing.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    #[derive(PrimeField)]
    #[PrimeFieldModulus = "23"]
    #[PrimeFieldGenerator = "11"]
    #[PrimeFieldReprEndianness = "little"]
    struct F([u64; 1]);

    // configuration for low prime number field
    let gmimc = gmimc_erf::<F, 6> {
        capacity: 3,
        words: 2,
        round: 121,
        _field: std::marker::PhantomData::<F>,
    };

    let sequencial_points: Vec<(u64, u64)> = {
        let mut points_slices: [(u64, u64); 512] = [(0, 0); 512];
        for x in 0..512 {
            let y = gmimc
                .get_hash_output(&[x as u128, 0, 0, 0])
                .into_iter()
                .fold(0, |acc, x| (acc + x) % 23 as u128);
            points_slices[x] = (x as u64, y as u64);
        }
        points_slices.to_vec()
    };

    let mut scatter_ctx = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u64..30u64, 0u64..30u64)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(
        sequencial_points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 1, BLACK.filled())),
    )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
