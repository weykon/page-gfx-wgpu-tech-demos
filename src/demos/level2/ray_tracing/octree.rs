#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
enum Octant {
    NE, // Northeast
    NW, // Northwest
    SE, // Southeast
    SW, // Southwest
    E,  // East
    W,  // West
    S,  // South
    C,  // Center (中心)
}

#[derive(Debug, Clone)]
struct OctreeNode {
    bounds: Bounds,
    points: Vec<Point>,
    children: [Option<Box<OctreeNode>>; 8], // 八个子节点
}

impl OctreeNode {
    fn new(bounds: Bounds) -> Self {
        OctreeNode {
            bounds,
            points: Vec::new(),
            children: std::array::from_fn(|_| None),
        }
    }
}

#[derive(Debug, Clone)]
struct Bounds {
    min: Point,
    max: Point,
}

impl Bounds {
    fn new(min: Point, max: Point) -> Self {
        Bounds { min, max }
    }

    fn center(&self) -> Point {
        Point {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }

    fn size(&self) -> f64 {
        (self.max.x - self.min.x)
            .abs()
            .max((self.max.y - self.min.y).abs())
            .max((self.max.z - self.min.z).abs())
    }
}

struct Octree {
    root: Option<Box<OctreeNode>>,
    capacity: usize, // 每个节点最多容纳的点数
}

impl Octree {
    fn new(capacity: usize) -> Self {
        Octree {
            root: None,
            capacity,
        }
    }

    fn insert(&mut self, point: Point) {
        if self.root.is_none() {
            let bounds = Bounds::new(
                Point {
                    x: point.x - 1.0,
                    y: point.y - 1.0,
                    z: point.z - 1.0,
                },
                Point {
                    x: point.x + 1.0,
                    y: point.y + 1.0,
                    z: point.z + 1.0,
                },
            );
            self.root = Some(Box::new(OctreeNode::new(bounds)));
        }

        if let Some(root) = &mut self.root {
            Self::insert_recursive(root, point, self.capacity);
        }
    }
    
    fn insert_recursive(node_ref: &mut Box<OctreeNode>, point: Point, capacity: usize) {
        // 1. 如果节点已满，则分裂
        if node_ref.points.len() >= capacity {
            Self::subdivide(node_ref, capacity);
        }

        // 2. 将点插入到合适的子八分块中
        let center = node_ref.bounds.center();
        let octant = match (point.x >= center.x, point.y >= center.y, point.z >= center.z) {
            (false, false, false) => Octant::NW,
            (false, false, true) => Octant::SW,
            (false, true, false) => Octant::NE,
            (false, true, true) => Octant::SE,
            (true, false, false) => Octant::W,
            (true, false, true) => Octant::E,
            (true, true, false) => Octant::S,
            (true, true, true) => Octant::C,
        };

        let idx = match octant {
            Octant::NW => 0,
            Octant::SW => 1,
            Octant::NE => 2,
            Octant::SE => 3,
            Octant::W => 4,
            Octant::E => 5,
            Octant::S => 6,
            Octant::C => 7,
        };

        if node_ref.children[idx].is_some() {
            if let Some(child) = &mut node_ref.children[idx] {
                Self::insert_recursive(child, point, capacity);
            }
        } else {
            node_ref.points.push(point);
        }
    }

    fn subdivide(node: &mut Box<OctreeNode>, capacity: usize) {
        let center = node.bounds.center();
        
        // 创建八个子节点
        for i in 0..8 {
            let min = &node.bounds.min;
            let max = &node.bounds.max;
            
            let bounds = match i {
                // NW: 左下前
                0 => Bounds::new(
                    Point { x: min.x, y: min.y, z: min.z },
                    Point { x: center.x, y: center.y, z: center.z },
                ),
                // SW: 左下后
                1 => Bounds::new(
                    Point { x: min.x, y: min.y, z: center.z },
                    Point { x: center.x, y: center.y, z: max.z },
                ),
                // NE: 左上前
                2 => Bounds::new(
                    Point { x: min.x, y: center.y, z: min.z },
                    Point { x: center.x, y: max.y, z: center.z },
                ),
                // SE: 左上后
                3 => Bounds::new(
                    Point { x: min.x, y: center.y, z: center.z },
                    Point { x: center.x, y: max.y, z: max.z },
                ),
                // W: 右下前
                4 => Bounds::new(
                    Point { x: center.x, y: min.y, z: min.z },
                    Point { x: max.x, y: center.y, z: center.z },
                ),
                // E: 右下后
                5 => Bounds::new(
                    Point { x: center.x, y: min.y, z: center.z },
                    Point { x: max.x, y: center.y, z: max.z },
                ),
                // S: 右上前
                6 => Bounds::new(
                    Point { x: center.x, y: center.y, z: min.z },
                    Point { x: max.x, y: max.y, z: center.z },
                ),
                // C: 右上后
                7 => Bounds::new(
                    Point { x: center.x, y: center.y, z: center.z },
                    Point { x: max.x, y: max.y, z: max.z },
                ),
                _ => unreachable!(),
            };
            
            node.children[i] = Some(Box::new(OctreeNode::new(bounds)));
        }

        // 将当前节点中的点移动到子节点中
        let points = std::mem::take(&mut node.points);
        for point in points {
            Self::insert_recursive(node, point, capacity);
        }
    }

    fn print_tree(&self) {
        if let Some(root) = &self.root {
            self.print_node(root, 0);
        }
    }

    fn print_node(&self, node: &OctreeNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}Node at depth {}", indent, depth);
        println!("{}Bounds: min({:.1}, {:.1}, {:.1}), max({:.1}, {:.1}, {:.1})",
            indent, 
            node.bounds.min.x, node.bounds.min.y, node.bounds.min.z,
            node.bounds.max.x, node.bounds.max.y, node.bounds.max.z
        );
        
        println!("{}Points: {}", indent, node.points.len());
        for (i, point) in node.points.iter().enumerate() {
            println!("{}  Point {}: ({:.1}, {:.1}, {:.1})", 
                indent, i, point.x, point.y, point.z);
        }
        
        for (i, child) in node.children.iter().enumerate() {
            if let Some(child_node) = child {
                println!("{}Child {} (octant {:?}):", indent, i, match i {
                    0 => Octant::NW,
                    1 => Octant::SW,
                    2 => Octant::NE,
                    3 => Octant::SE,
                    4 => Octant::W,
                    5 => Octant::E,
                    6 => Octant::S,
                    7 => Octant::C,
                    _ => unreachable!(),
                });
                self.print_node(child_node, depth + 1);
            }
        }
    }
}

#[test]
fn test() {
    let mut octree = Octree::new(4);

    octree.insert(Point {
        x: 0.5,
        y: 0.5,
        z: 0.5,
    });
    octree.insert(Point {
        x: 1.5,
        y: 2.5,
        z: 0.8,
    });
    octree.insert(Point {
        x: 3.2,
        y: 1.1,
        z: 2.9,
    });
    octree.insert(Point {
        x: 0.2,
        y: 0.7,
        z: 1.4,
    });
    octree.insert(Point {
        x: 2.8,
        y: 3.5,
        z: 1.6,
    });

    octree.print_tree();
}
