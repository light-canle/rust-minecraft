// Voxel을 위한 특수한 Raycast(정수 기준 raycast)
use nalgebra_glm::{floor, IVec3, Vec3};
use num_traits::float::FloatCore;

// direction must be normalized
pub fn raycast<T>(
    get_voxel: &dyn Fn(i32, i32, i32) -> Option<T>,
    origin: &Vec3,
    direction: &Vec3,
    distance: f32,
) -> Option<(T, IVec3)> {
    let mut t = 0.0f32;
    // 카메라 위치를 정수로 가져옴
    let mut i = floor(&origin).map(|x| x as i32);
    let step = direction.map(|x| if x > 0f32 { 1 } else { -1 });
    // t 변화량
    let t_delta = direction.map(|x| (1.0 / x).abs());
    // 현재 origin에서 다음 정수 거리까지 거리(zip_zip_map은 3개를 묶은 뒤 클로저를 적용)
    let dist = origin.zip_zip_map(&i, &step, |p, i, s| {
        if s > 0 {
            i as f32 + 1.0 - p // 양수인 경우
        } else {
            p - i as f32 // 음수인 경우
        }
    });
    let mut t_max = t_delta.zip_map(&dist, |t, d| {
        if t.is_finite() {
            t * d
        } else {
            // t = 0으로 역수인 dt가 무한한 경우
            f32::infinity()
        }
    });

    let mut hit_pos = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_norm = IVec3::new(0, 0, 0);

    let mut stepped_index = -1;
    while t <= distance {
        // exit check
        if let Some(voxel) = get_voxel(i.x, i.y, i.z) {
            // origin에 있는 블록을 가져옴
            hit_pos = origin.zip_map(&direction, |p, d| p + t * d);
            // 충돌이 발생함 - 충돌한 면의 Normal Vector를 반환
            if stepped_index == 0 {
                hit_norm[0] = -step.x;
            }
            if stepped_index == 1 {
                hit_norm[1] = -step.y;
            }
            if stepped_index == 2 {
                hit_norm[2] = -step.z;
            }
            return Some((voxel, hit_norm));
        }

        // advance t to next nearest voxel boundary(가장 가까운 정수 위치가 잡히도록 t를 조정해준다.)
        // x, y, z성분 중 최소를 구함
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                // 가장 작은 성분이 가리키는 방향으로 이동
                i.x += step.x;
                t = t_max.x;
                t_max.x += t_delta.x;
                stepped_index = 0;
            } else {
                i.z += step.z;
                t = t_max.z;
                t_max.z += t_delta.z;
                stepped_index = 2;
            }
        } else {
            if (t_max.y < t_max.z) {
                i.y += step.y;
                t = t_max.y;
                t_max.y += t_delta.y;
                stepped_index = 1;
            } else {
                i.z += step.z;
                t = t_max.z;
                t_max.z += t_delta.z;
                stepped_index = 2;
            }
        }
    }

    // no voxel hit found - return None
    hit_pos = origin.zip_map(&direction, |p, d| p + t * d);

    None
}
