use criterion::{criterion_group, criterion_main, Criterion};

use egui_demo_lib::LOREM_IPSUM_LONG;

pub fn criterion_benchmark(c: &mut Criterion) {
    let raw_input = egui::RawInput::default();

    {
        let mut ctx = egui::CtxRef::default();
        let mut demo_windows = egui_demo_lib::DemoWindows::default();

        // The most end-to-end benchmark.
        c.bench_function("demo_with_tesselate__realistic", |b| {
            b.iter(|| {
                ctx.begin_frame(raw_input.clone());
                demo_windows.ui(&ctx);
                let (_, shapes) = ctx.end_frame();
                ctx.tessellate(shapes)
            })
        });

        c.bench_function("demo_no_tesselate", |b| {
            b.iter(|| {
                ctx.begin_frame(raw_input.clone());
                demo_windows.ui(&ctx);
                ctx.end_frame()
            })
        });

        ctx.begin_frame(raw_input.clone());
        demo_windows.ui(&ctx);
        let (_, shapes) = ctx.end_frame();
        c.bench_function("demo_only_tessellate", |b| {
            b.iter(|| ctx.tessellate(shapes.clone()))
        });
    }

    if false {
        let mut ctx = egui::CtxRef::default();
        ctx.memory().set_everything_is_visible(true); // give us everything
        let mut demo_windows = egui_demo_lib::DemoWindows::default();
        c.bench_function("demo_full_no_tesselate", |b| {
            b.iter(|| {
                ctx.begin_frame(raw_input.clone());
                demo_windows.ui(&ctx);
                ctx.end_frame()
            })
        });
    }

    {
        let mut ctx = egui::CtxRef::default();
        ctx.begin_frame(raw_input.clone());
        let mut ui = egui::Ui::__test();
        c.bench_function("label &str", |b| {
            b.iter(|| {
                ui.label("the quick brown fox jumps over the lazy dog");
            })
        });
        c.bench_function("label format!", |b| {
            b.iter(|| {
                ui.label("the quick brown fox jumps over the lazy dog".to_owned());
            })
        });
    }

    {
        let pixels_per_point = 1.0;
        let wrap_width = 512.0;
        let text_style = egui::TextStyle::Body;
        let fonts = egui::epaint::text::Fonts::from_definitions(
            pixels_per_point,
            egui::FontDefinitions::default(),
        );
        let font = &fonts[text_style];
        c.bench_function("text_layout_uncached", |b| {
            b.iter(|| font.layout_multiline(LOREM_IPSUM_LONG.to_owned(), wrap_width))
        });
        c.bench_function("text_layout_cached", |b| {
            b.iter(|| fonts.layout_multiline(text_style, LOREM_IPSUM_LONG.to_owned(), wrap_width))
        });

        let galley = font.layout_multiline(LOREM_IPSUM_LONG.to_owned(), wrap_width);
        let mut tessellator = egui::epaint::Tessellator::from_options(Default::default());
        let mut mesh = egui::epaint::Mesh::default();
        c.bench_function("tessellate_text", |b| {
            b.iter(|| {
                let fake_italics = false;
                tessellator.tessellate_text(
                    fonts.texture().size(),
                    egui::Pos2::ZERO,
                    &galley,
                    egui::Color32::WHITE,
                    fake_italics,
                    &mut mesh,
                );
                mesh.clear();
            })
        });
    }

    {
        fn points_plot(num_of_items: u32, num_of_points: u32) -> egui::plot::Plot {
            let mut plot = egui::plot::Plot::new("Benchmark");
            for item_index in 0..num_of_items {
                plot = plot.points(egui::plot::Points::new(egui::plot::Values::from_values(
                    (0..num_of_points)
                        .map(move |p| egui::plot::Value::new(p as f32, (p + item_index) as f32))
                        .collect(),
                )));
            }
            plot
        }

        fn lines_plot(num_of_items: u32, num_of_points: u32) -> egui::plot::Plot {
            let mut plot = egui::plot::Plot::new("Benchmark");
            for item_index in 0..num_of_items {
                plot = plot.line(egui::plot::Line::new(
                    egui::plot::Values::from_explicit_callback(
                        move |x| x + item_index as f64,
                        f64::NEG_INFINITY..=f64::INFINITY,
                        num_of_points as usize,
                    ),
                ));
            }
            plot
        }

        fn bars_plot(num_of_items: u32, num_of_bars: u32) -> egui::plot::Plot {
            let mut plot = egui::plot::Plot::new("Benchmark");
            for item_index in 0..num_of_items {
                plot = plot.barchart(egui::plot::BarChart::new(
                    (0..num_of_bars)
                        .map(|i| egui::plot::Bar::new(i as f64, 1.0).base_offset(item_index as f64))
                        .collect(),
                ));
            }
            plot
        }

        let mut ctx = egui::CtxRef::default();
        let mut group = c.benchmark_group("plots");
        group
            .sample_size(1000)
            .measurement_time(std::time::Duration::from_secs(15));
        let mut run_plot_bench =
            |name: &str,
             num_of_items: u32,
             item_size: u32,
             plot: fn(u32, u32) -> egui::plot::Plot| {
                group.bench_function(name, |b| {
                    b.iter(|| {
                        let mut input = raw_input.clone();
                        input
                            .events
                            .push(egui::Event::PointerMoved(egui::Pos2::new(250.0, 250.0)));
                        ctx.begin_frame(input);

                        egui::CentralPanel::default().show(&ctx, |ui| {
                            ui.add(plot(num_of_items, item_size));
                        });

                        ctx.end_frame()
                    })
                });
            };

        let size_matrix = [
            [10, 10],
            [10, 100],
            [10, 1000],
            [10, 10000],
            [100, 10],
            [100, 100],
            [100, 1000],
            [1000, 10],
            [1000, 100],
            [10000, 10],
        ];
        for [num_of_items, item_size] in size_matrix {
            run_plot_bench(
                &format!("plot_{}x{}_points", num_of_items, item_size),
                num_of_items,
                item_size,
                points_plot,
            );
            run_plot_bench(
                &format!("plot_{}x{}_lines", num_of_items, item_size),
                num_of_items,
                item_size,
                lines_plot,
            );
            run_plot_bench(
                &format!("plot_{}x{}_bars", num_of_items, item_size),
                num_of_items,
                item_size,
                bars_plot,
            );
        }
        group.finish();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
