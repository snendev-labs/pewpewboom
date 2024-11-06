use noise::{utils::*, Fbm, Perlin};

pub fn test() {
    let fbm = Fbm::<Perlin>::default();

    let plane = PlaneMapBuilder::new(fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build();
}
