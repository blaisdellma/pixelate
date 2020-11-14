use rand::prelude::*;
use rand::seq::index::sample;

#[derive(Default,Debug,Copy,Clone)]
pub struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self{
        Vec4 {x,y,z,w}
    }

    pub fn get(&self) -> (f32,f32,f32,f32) {
        (self.x,self.y,self.z,self.w)
    }

    fn dist2(&self, other: &Vec4) -> f32 {
        (self.x-other.x)*(self.x-other.x) +
        (self.y-other.y)*(self.y-other.y) +
        (self.z-other.z)*(self.z-other.z) +
        (self.w-other.w)*(self.w-other.w)
    }

    fn classify(&self, others: &Vec<Vec4>) -> usize {
        others.iter().enumerate().map(|(i, &other)| (i,self.dist2(&other))).fold((0,1000.0),|(i,idist2),(j,jdist2)|{
            if idist2 < jdist2 {
                (i,idist2)
            } else {
                (j,jdist2)
            }
        }).0
    }

}

fn avg(vec4s: &Vec<&Vec4>) -> Vec4 {
    let n = vec4s.len() as f32;
    if n == 0.0 {
        return Vec4::default();
    }
    let mut v = vec4s.iter().fold(Vec4::default(),|v1,v2| {
        Vec4::new(v1.x + v2.x, v1.y + v2.y, v1.z + v2.z, v1.w + v2.w,)
    });
    v.x /= n;
    v.y /= n;
    v.z /= n;
    v.w /= n;
    v
}

fn get_new_means(data: &Vec<Vec4>, means: &Vec<Vec4>) -> (f32,Vec<Vec4>) {
    let mut classification = Vec::new();
    classification.resize(means.len(),Vec::<&Vec4>::new());
    let mut total = 0.0;
    for vec in data {
        let i = vec.classify(means);
        classification[i].push(vec);
        total += vec.dist2(&means[i]);
    }
    println!("DIST2 total: {}", total);
    (total, classification.iter().map(|vecs| avg(vecs)).collect())

}



pub fn kmeans(data_in: Vec<Vec4>, k_num: usize) -> Vec<Vec4> {
    println!("KMEANS");

    let means: Vec<Vec4> = sample(&mut thread_rng(),data_in.len(),k_num).iter().map(|i| data_in[i]).collect();

    let (mut err, mut means) = get_new_means(&data_in,&means);

    for _ in 0..200 {
        let (err_new, means_new) = get_new_means(&data_in,&means);
        if (err-err_new)/err < 0.001 {
            break
        }
        err = err_new;
        means = means_new;
    }

    data_in.iter().map(|v| means[v.classify(&means)]).collect()


}
