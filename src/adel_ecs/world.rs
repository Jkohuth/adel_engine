use crate::adel_tools::print_type_of;
use std::any::TypeId;
use std::cell::{Ref, RefCell, RefMut};

#[allow(unused_imports)]
use std::collections::HashMap;

pub trait Component {
    fn component_as_any(&self) -> &dyn std::any::Any;
    fn component_as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
}

// Every component type needs to be known at run time and last the duration of the program
// Implemented for a mutable vectors that could contain the component
// All component vectors have the same size
impl<T: 'static> Component for RefCell<Vec<Option<T>>> {
    // Borrow Vec of EntityIds that represent a component
    fn component_as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    // Mutability borrow Vec of EntityIds taht represent a component
    fn component_as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
    // Push empty into that vector location
    fn push_none(&mut self) {
        self.get_mut().push(None)
    }
}
// Resources are unique objects (one per application) that may be accessed from world at any time. The idea
// behind resources was not to have multiple components that share the same data. For example, user input may
// effect multiple different systems and components but there is only one set of it. Each component doesn't need
// the duplicated data.
pub trait Resource {
    fn resource_as(&self) -> &dyn std::any::Any;
    fn resource_as_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> Resource for RefCell<T> {
    fn resource_as(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    fn resource_as_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

// Need to define world that will hold all the entities and components
pub struct World {
    entities_count: usize,
    components: Vec<Box<dyn Component>>,
    resources: HashMap<TypeId, Box<dyn Resource>>,
    dt: f32,
}

impl World {
    pub fn new() -> Self {
        World {
            entities_count: 0,
            components: Vec::new(),
            resources: HashMap::new(),
            dt: 0.0,
        }
    }
    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component in self.components.iter_mut() {
            component.push_none();
        }
        self.entities_count += 1;
        entity_id
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Iterate through all of the available components until one matches the provided componentType
        for component_vec in self.components.iter_mut() {
            if let Some(component_vec) = component_vec
                .component_as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                // For that Component Vector make an entry switch None for Some at location EntityId
                component_vec.borrow_mut()[entity] = Some(component);
                return;
            }
        }
        let mut new_component: Vec<Option<ComponentType>> = Vec::with_capacity(self.entities_count);

        // Populate the new component with None for all entites as it was just created
        for _ in 0..self.entities_count {
            new_component.push(None);
        }

        // Add Entity to newly created Component
        new_component[entity] = Some(component);

        // Append new Component to the Vector of pointers (Box) stored in world
        self.components.push(Box::new(RefCell::new(new_component)));
    }
    // Creates a function to insert an existing component Vector into World
    pub fn insert_component<ComponentType: 'static>(
        &mut self,
        component: Vec<Option<ComponentType>>,
    ) {
        // All the component Vectors need to be the same length, if we have a mismatched number of entities, fail
        // Note: Arrays indexed at 0
        assert_eq!(self.entities_count, component.len());
        self.components.push(Box::new(RefCell::new(component)));
    }
    pub fn borrow_component_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.components.iter() {
            if let Some(component_vec) = component_vec
                .component_as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
    pub fn borrow_component<ComponentType: 'static>(
        &self,
    ) -> Option<Ref<Vec<Option<ComponentType>>>> {
        for component_vec in self.components.iter() {
            if let Some(component_vec) = component_vec
                .component_as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow());
            }
        }
        None
    }
    // At this time this method is limited to one entry per type
    // TODO: Make key/value pair capable of handling multiple inputs of the same type (ie change the key)
    pub fn insert_resource<R: 'static>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        let boxed_resource = Box::new(RefCell::new(resource));
        //print_type_of(&boxed_resource);
        //self.resources.insert(type_id, (&mut resource as *mut R).cast::<u8>());
        self.resources.insert(type_id, boxed_resource);
    }
    // The + 'static lets the program know that the type provided will be valid for the duration of the program and not a reference
    pub fn get_resource<R: 'static>(&self) -> Option<Ref<R>> {
        let type_id = TypeId::of::<R>();
        let boxed_resource = self.resources.get(&type_id).unwrap();
        if let Some(resource) = boxed_resource.resource_as().downcast_ref::<RefCell<R>>() {
            return Some(resource.borrow());
        } else {
            //log::info!("Failed to Downcast Value");
            return None;
        }
    }
    pub fn get_resource_mut<R: 'static>(&self) -> Option<RefMut<R>> {
        let type_id = TypeId::of::<R>();
        let box_resource = match self.resources.get(&type_id) {
            Some(box_resource) => box_resource,
            None => {
                log::info!("ERROR: No resource found {:?}", print_type_of(&type_id));
                return None;
            }
        };
        if let Some(resource) = box_resource.resource_as().downcast_ref::<RefCell<R>>() {
            return Some(resource.borrow_mut());
        } else {
            //log::info!("Failed to Downcast Value Mut");
            return None;
        }
    }

    pub fn update_dt(&mut self, dt: f32) {
        self.dt = dt;
    }
    pub fn get_dt(&self) -> f32 {
        self.dt
    }

    /*
        Old Resource Code attempting to Use Raw pointers, led to Errors when the memory being allocated was on the stack
        and dereferencing the pointers (whose values had been dropped by that time) led to a segmentation fault. I don't
        want to remove this code as I want to revisit this at a later time and perhaps explain the various differences
        between Box<dyn TraitObject> vs *mut u8 pointer that is casted to the correct type

    // Insert the resource in the World Struct as a *mut u8 in order to include various types
    // that need to be stored
    pub fn insert_resource<R: Resource>(&mut self, mut resource: R) -> TypeId {
        let type_id = TypeId::of::<R>();
        let ptr = (&mut resource as *mut R).cast::<u8>();
        self.resources.insert(type_id, ptr);
        //self.resources.insert(type_id, (&mut resource as *mut R).cast::<u8>());
        log::info!("Pointer Address {:?}", ptr);
        type_id
    }

    // Unsafe used to dereference raw pointer.
    pub fn get_resource<R: Resource>(&mut self) -> Option<&R> {
        let type_id = TypeId::of::<R>();
        let option_resource = self.resources.get(&type_id);
        // Safe: Key (TypeId) present in HashMap
        unsafe {
            match option_resource {
                Some(resource) => {
                    log::info!("Get Resource {:?}", resource);
                    return Some(&*(resource).cast::<R>());
                }
                None => None,
            }
        }
    }

    // Unsafe used to dereference raw pointer.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let type_id = TypeId::of::<R>();
        let option_resource = self.resources.get(&type_id);
        // Safe: Key (TypeId) present in HashMap
        unsafe {
            match option_resource {
                Some(resource) => {
                    log::info!("Get Resource Mut {:?}", resource);
                    return Some(&mut *(resource).cast::<R>());
                }
                None => None,
            }
        }
    }
    */
}
