use gametesting::ComponentVec;
use std::borrow::BorrowMut;
use std::cell::{RefMut, Ref, RefCell};
use std::collections::{BinaryHeap, HashSet, BTreeSet};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::ops::RangeBounds;
use std::hash::{Hash, Hasher};

use crate::lib::Collider;
use crate::worldgen::Tile;


pub struct EContainer {
    component_vecs: Vec<Box<dyn ComponentVec>>,
    pub free_entities: BinaryHeap<Reverse<usize>>,
    entity_subsets: HashMap<Vec<TypeId>, BTreeSet<usize>>,
    entity_count: usize,
}

impl EContainer {
    pub fn new() -> Self {
        Self {
            component_vecs: Vec::new(),
            free_entities: BinaryHeap::new(),
            entity_subsets: HashMap::new(),
            entity_count: 0,
        }
    }
    
    pub fn new_entity(&mut self) -> usize {
        // Sets the new entity to the smallest free index if one is available
        if let Some(Reverse(free_id)) = self.free_entities.pop() {
            return free_id
        }
        
        // If no free slots available, pushes to end of vec
        let entity_id = self.entity_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entity_count += 1;
        return entity_id
    }
    
    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
    
    pub fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<Ref<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow());
            }
        }
        None
    }
    
    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }
        // No matching component storage exists yet, so we have to make one.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entity_count);
    
        // All existing entities don't have this component, so we give them `None`
        for _ in 0..self.entity_count {
            new_component_vec.push(None);
        }
    
        // Give this Entity the Component.
        new_component_vec[entity] = Some(component);
        self.component_vecs.push(Box::new(RefCell::new(new_component_vec)));   
    }
    
    pub fn remove_entity_component<ComponentType: 'static>(
        &mut self,
        entity: usize,
        _component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = None;
                return;
            }
        }
        //TODO: Remove blank componentvecs if they become blank as a result of component removal
        //      Also mark as deleted if completely blank    // Or maybe not?
    }
    
    pub fn remove_entity(
        &mut self,
        entity: usize,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.set_none(entity);
        }
        self.free_entities.push(Reverse(entity));
    }
    
    //maybe one day
    /* 
    pub fn query<T: 'static>(&mut self, query: Vec<TypeId>) -> Vec<Vec<&mut T>> {
        'sets: for set in self.entity_subsets.keys() {
            for type_id in &query {
                if !set.contains(type_id) {
                    continue 'sets;
                }
            }
            let mut queried_vecs = Vec::new();

            for type_id in &query {
                let mut queried_component_vec = Vec::new();
                for component_vec in self.component_vecs.iter_mut() {
                    if let Some(component_vec) = component_vec
                        .as_any_mut()
                        .downcast_mut::<RefCell<Vec<Option<T>>>>()
                    {
                        let indexes = self.entity_subsets[set];
                        for index in indexes {
                            queried_component_vec.push(component_vec.get_mut()[index].as_mut().unwrap())
                        }
                    }
                }
                queried_vecs.push(queried_component_vec);
            }
            
        }
        return Vec::new();
    }
    */
    
    pub fn from(coords: &(usize, usize), tile: &Tile) -> EContainer {
        
        return EContainer::new();
    }
}