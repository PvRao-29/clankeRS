use clankers_core::RobotResult;
use ndarray::Array2;

use crate::layout::DType;

/// Point cloud tensor (Nx3 or Nx4 points).
#[derive(Debug, Clone)]
pub struct PointCloudTensor {
    pub points: Array2<f32>,
    pub dtype: DType,
    pub frame_id: String,
}

impl PointCloudTensor {
    pub fn from_xyz(points: Vec<[f32; 3]>) -> RobotResult<Self> {
        let n = points.len();
        let mut array = Array2::<f32>::zeros((n, 3));
        for (i, [x, y, z]) in points.into_iter().enumerate() {
            array[[i, 0]] = x;
            array[[i, 1]] = y;
            array[[i, 2]] = z;
        }
        Ok(Self {
            points: array,
            dtype: DType::F32,
            frame_id: "lidar".to_string(),
        })
    }

    pub fn num_points(&self) -> usize {
        self.points.nrows()
    }

    pub fn to_vec(&self) -> Vec<f32> {
        self.points.iter().copied().collect()
    }
}
