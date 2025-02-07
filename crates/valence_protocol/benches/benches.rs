use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use valence_nbt::{compound, List};
use valence_protocol::block::{BlockKind, BlockState, PropName, PropValue};
use valence_protocol::packets::s2c::play::{
    ChunkDataAndUpdateLight, ChunkDataAndUpdateLightEncode, SetTabListHeaderAndFooter, SpawnEntity,
};
use valence_protocol::text::Color;
use valence_protocol::{
    encode_packet, encode_packet_compressed, ByteAngle, Decode, Encode, ItemKind,
    LengthPrefixedArray, PacketDecoder, PacketEncoder, TextFormat, VarInt,
};

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5)).confidence_level(0.99);
    targets = blocks, packets, var_int, decode_array
}
criterion_main!(benches);

fn blocks(c: &mut Criterion) {
    let states = BlockKind::ALL.map(BlockKind::to_state);

    c.bench_function("BlockState::from_kind", |b| {
        b.iter(|| {
            for kind in black_box(BlockKind::ALL) {
                black_box(BlockState::from_kind(kind));
            }
        });
    });

    c.bench_function("BlockState::to_kind", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.to_kind());
            }
        });
    });

    c.bench_function("BlockState::get", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.get(PropName::Note));
            }
        });
    });

    c.bench_function("BlockState::set", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.set(PropName::Note, PropValue::Didgeridoo));
            }
        });
    });

    c.bench_function("BlockState::is_liquid", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.is_liquid());
            }
        });
    });

    c.bench_function("BlockState::is_opaque", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.is_opaque());
            }
        })
    });

    c.bench_function("BlockState::is_replaceable", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.is_replaceable());
            }
        })
    });

    c.bench_function("BlockState::luminance", |b| {
        b.iter(|| {
            for state in black_box(states) {
                black_box(state.luminance());
            }
        })
    });

    c.bench_function("BlockKind::to_item_kind", |b| {
        b.iter(|| {
            for kind in black_box(BlockKind::ALL) {
                black_box(kind.to_item_kind());
            }
        });
    });

    c.bench_function("BlockKind::from_item_kind", |b| {
        b.iter(|| {
            for kind in black_box(ItemKind::ALL) {
                black_box(BlockKind::from_item_kind(kind));
            }
        });
    });
}

fn packets(c: &mut Criterion) {
    let mut encoder = PacketEncoder::new();

    const BLOCKS_AND_BIOMES: [u8; 2000] = [0x80; 2000];
    const SKY_LIGHT_ARRAYS: [LengthPrefixedArray<u8, 2048>; 26] =
        [LengthPrefixedArray([0xff; 2048]); 26];

    let chunk_data_packet = ChunkDataAndUpdateLightEncode {
        chunk_x: 123,
        chunk_z: 456,
        heightmaps: &compound! {
            "MOTION_BLOCKING" => List::Long(vec![123; 256]),
        },
        blocks_and_biomes: BLOCKS_AND_BIOMES.as_slice(),
        block_entities: &[],
        trust_edges: false,
        sky_light_mask: &[],
        block_light_mask: &[],
        empty_sky_light_mask: &[],
        empty_block_light_mask: &[],
        sky_light_arrays: SKY_LIGHT_ARRAYS.as_slice(),
        block_light_arrays: &[],
    };

    let tab_list_header_footer_packet = SetTabListHeaderAndFooter {
        header: ("this".italic() + " is the " + "header".bold().color(Color::RED)).into(),
        footer: ("this".italic()
            + " is the "
            + "footer".bold().color(Color::BLUE)
            + ". I am appending some extra text so that the packet goes over the compression \
               threshold.")
            .into(),
    };

    let spawn_entity_packet = SpawnEntity {
        entity_id: VarInt(1234),
        object_uuid: Default::default(),
        kind: VarInt(5),
        position: [123.0, 456.0, 789.0],
        pitch: ByteAngle(200),
        yaw: ByteAngle(100),
        head_yaw: ByteAngle(50),
        data: VarInt(i32::MIN),
        velocity: [12, 34, 56],
    };

    c.bench_function("encode_chunk_data", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder.append_packet(&chunk_data_packet).unwrap();

            black_box(encoder);
        });
    });

    c.bench_function("encode_tab_list_header_footer", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder
                .append_packet(&tab_list_header_footer_packet)
                .unwrap();

            black_box(encoder);
        });
    });

    c.bench_function("encode_spawn_entity", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder.append_packet(&spawn_entity_packet).unwrap();

            black_box(encoder);
        });
    });

    encoder.set_compression(Some(256));

    c.bench_function("encode_chunk_data_compressed", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder.append_packet(&chunk_data_packet).unwrap();

            black_box(encoder);
        });
    });

    c.bench_function("encode_tab_list_header_footer_compressed", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder
                .append_packet(&tab_list_header_footer_packet)
                .unwrap();

            black_box(encoder);
        });
    });

    c.bench_function("encode_spawn_entity_compressed", |b| {
        b.iter(|| {
            let encoder = black_box(&mut encoder);

            encoder.clear();
            encoder.append_packet(&spawn_entity_packet).unwrap();

            black_box(encoder);
        });
    });

    let mut decoder = PacketDecoder::new();
    let mut packet_buf = vec![];

    encode_packet(&mut packet_buf, &chunk_data_packet).unwrap();

    c.bench_function("decode_chunk_data", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder
                .try_next_packet::<ChunkDataAndUpdateLight>()
                .unwrap();

            black_box(decoder);
        });
    });

    packet_buf.clear();
    encode_packet(&mut packet_buf, &tab_list_header_footer_packet).unwrap();

    c.bench_function("decode_tab_list_header_footer", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder
                .try_next_packet::<SetTabListHeaderAndFooter>()
                .unwrap();

            black_box(decoder);
        });
    });

    packet_buf.clear();
    encode_packet(&mut packet_buf, &spawn_entity_packet).unwrap();

    c.bench_function("decode_spawn_entity", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder.try_next_packet::<SpawnEntity>().unwrap();

            black_box(decoder);
        });
    });

    decoder.set_compression(true);

    let mut scratch = vec![];

    packet_buf.clear();
    encode_packet_compressed(&mut packet_buf, &chunk_data_packet, 256, &mut scratch).unwrap();

    c.bench_function("decode_chunk_data_compressed", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder
                .try_next_packet::<ChunkDataAndUpdateLight>()
                .unwrap();

            black_box(decoder);
        });
    });

    packet_buf.clear();
    encode_packet_compressed(
        &mut packet_buf,
        &tab_list_header_footer_packet,
        256,
        &mut scratch,
    )
    .unwrap();

    c.bench_function("decode_tab_list_header_footer_compressed", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder
                .try_next_packet::<SetTabListHeaderAndFooter>()
                .unwrap();

            black_box(decoder);
        });
    });

    packet_buf.clear();
    encode_packet_compressed(&mut packet_buf, &spawn_entity_packet, 256, &mut scratch).unwrap();

    c.bench_function("decode_spawn_entity_compressed", |b| {
        b.iter(|| {
            let decoder = black_box(&mut decoder);

            decoder.queue_slice(&packet_buf);
            decoder.try_next_packet::<SpawnEntity>().unwrap();

            black_box(decoder);
        });
    });
}

fn var_int(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    c.bench_function("VarInt::encode", |b| {
        b.iter_with_setup(
            || rng.gen(),
            |i| {
                let i: i32 = black_box(i);

                let mut buf = [0; VarInt::MAX_SIZE];
                let _ = black_box(VarInt(i).encode(buf.as_mut_slice()));
            },
        );
    });

    c.bench_function("VarInt::decode", |b| {
        b.iter_with_setup(
            || {
                let mut buf = [0; VarInt::MAX_SIZE];
                VarInt(rng.gen()).encode(buf.as_mut_slice()).unwrap();
                buf
            },
            |buf| {
                let mut r = black_box(buf.as_slice());
                let _ = black_box(VarInt::decode(&mut r));
            },
        )
    });
}

fn decode_array(c: &mut Criterion) {
    let floats = [123.0, 456.0, 789.0];
    let mut buf = [0u8; 24];

    floats.encode(buf.as_mut_slice()).unwrap();

    c.bench_function("<[f64; 3]>::decode", |b| {
        b.iter(|| {
            let mut r = black_box(buf.as_slice());
            let _ = black_box(<[f64; 3]>::decode(&mut r));
        });
    });

    let bytes = [42; 4096];

    c.bench_function("<[u8; 4096]>::decode", |b| {
        b.iter(|| {
            let mut r = black_box(bytes.as_slice());
            let _ = black_box(<[u8; 4096]>::decode(&mut r));
        })
    });
}
