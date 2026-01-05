// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use taffy::prelude::*;
use moonwalk::{MoonWalk, ObjectId};
use glam::Vec2;
use crate::layout::Layout;

const Z_STEP: f32 = 0.00001;

pub struct WidgetTree {
    taffy: TaffyTree,
    bindings: HashMap<NodeId, ObjectId>,
    pub root: NodeId,
}

impl WidgetTree {
    pub fn new(root_style: Layout) -> Self {
        let mut taffy = TaffyTree::new();
        let root = taffy.new_leaf(root_style.build()).expect("Failed to create root");
        
        Self {
            taffy,
            bindings: HashMap::new(),
            root,
        }
    }

    pub fn new_node(&mut self, layout: Layout) -> NodeId {
        self.taffy.new_leaf(layout.build()).expect("Failed to create node")
    }

    pub fn bind(&mut self, node: NodeId, object: ObjectId) {
        self.bindings.insert(node, object);
    }
    
    pub fn unbind(&mut self, node: NodeId) {
        self.bindings.remove(&node);
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.taffy.add_child(parent, child).unwrap();
    }
    
    pub fn remove_node(&mut self, node: NodeId) {
        self.taffy.remove(node).unwrap();
        self.bindings.remove(&node);
    }

    pub fn set_style(&mut self, node: NodeId, layout: Layout) {
        self.taffy.set_style(node, layout.build()).unwrap();
    }

    pub fn compute_and_apply(&mut self, mw: &mut MoonWalk, width: f32, height: f32) {
        let space = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        
        self.taffy.compute_layout(self.root, space).unwrap();

        let mut z_counter = 0;
        
        self.sync_recursive(self.root, Vec2::ZERO, mw, &mut z_counter);
    }

    fn sync_recursive(
        &self, 
        node_id: NodeId, 
        parent_pos: Vec2, 
        mw: &mut MoonWalk, 
        z_counter: &mut usize
    ) {
        let layout = self.taffy.layout(node_id).unwrap();
        
        let local_pos = Vec2::new(layout.location.x, layout.location.y);
        let global_pos = parent_pos + local_pos;
        let size = Vec2::new(layout.size.width, layout.size.height);

        if let Some(&obj_id) = self.bindings.get(&node_id) {
            if mw.is_alive(obj_id) {
                mw.set_position(obj_id, global_pos);
                mw.set_size(obj_id, size);
                
                let z = (*z_counter as f32 * Z_STEP).min(0.999);
                mw.set_z_index(obj_id, z);
                
                *z_counter += 1;
            }
        }

        if let Ok(children) = self.taffy.children(node_id) {
            for child in children {
                self.sync_recursive(child, global_pos, mw, z_counter);
            }
        }
    }
}
