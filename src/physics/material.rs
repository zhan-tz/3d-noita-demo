use crate::world::chunk::Material;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialBehavior {
    Static,
    FallingSolid,
    Liquid,
    Gas,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    pub material: Material,
    pub behavior: MaterialBehavior,
    pub density: f32,
    pub flammable: bool,
    pub flow_speed: u32,
    pub lifetime: Option<u32>,
}

impl MaterialProperties {
    pub fn get(mat: Material) -> Self {
        match mat {
            Material::Air => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Gas,
                density: 0.0,
                flammable: false,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Stone => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Static,
                density: 2500.0,
                flammable: false,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Dirt => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Static,
                density: 1500.0,
                flammable: true,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Sand => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::FallingSolid,
                density: 1600.0,
                flammable: false,
                flow_speed: 1,
                lifetime: None,
            },
            Material::Water => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Liquid,
                density: 1000.0,
                flammable: false,
                flow_speed: 4,
                lifetime: None,
            },
            Material::Lava => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Liquid,
                density: 3000.0,
                flammable: false,
                flow_speed: 1,
                lifetime: None,
            },
            Material::Wood => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Static,
                density: 600.0,
                flammable: true,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Metal => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Static,
                density: 7800.0,
                flammable: false,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Ice => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Static,
                density: 917.0,
                flammable: false,
                flow_speed: 0,
                lifetime: None,
            },
            Material::Fire => MaterialProperties {
                material: mat,
                behavior: MaterialBehavior::Gas,
                density: 0.5,
                flammable: false,
                flow_speed: 2,
                lifetime: Some(10),
            },
        }
    }

    pub fn is_mobile(&self) -> bool {
        !matches!(self.behavior, MaterialBehavior::Static)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_materials_have_properties() {
        use std::mem::transmute;
        for i in 0..=9u8 {
            let mat: Material = unsafe { transmute(i) };
            let props = MaterialProperties::get(mat);
            assert_eq!(props.material, mat);
        }
    }

    #[test]
    fn test_density_ordering() {
        let fire = MaterialProperties::get(Material::Fire);
        let water = MaterialProperties::get(Material::Water);
        let sand = MaterialProperties::get(Material::Sand);
        let stone = MaterialProperties::get(Material::Stone);
        let metal = MaterialProperties::get(Material::Metal);
        assert!(fire.density < water.density);
        assert!(water.density < sand.density);
        assert!(sand.density < stone.density);
        assert!(stone.density < metal.density);
    }

    #[test]
    fn test_flammable_materials() {
        assert!(MaterialProperties::get(Material::Wood).flammable);
        assert!(MaterialProperties::get(Material::Dirt).flammable);
        assert!(!MaterialProperties::get(Material::Stone).flammable);
        assert!(!MaterialProperties::get(Material::Metal).flammable);
    }
}
