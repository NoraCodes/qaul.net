use crate::KnowledgeEngine;

/// Create a new KnowledgeEngine implementation with the given resolver function.
/// This function should translate synthetic (test) events into actual changes in the
/// state of the system under test.
pub fn new_knowledge_engine<System, Event, F>(
    resolve: F,
) -> impl KnowledgeEngine<System, Event, System>
where
    Event: Clone,
    F: Fn(Event, System) -> System + 'static,
{
    KnowledgeEngineImpl {
        prologue: Vec::new(),
        events: Vec::new(),
        epilogue: Vec::new(),
        resolve: Box::new(resolve),
    }
}

struct KnowledgeEngineImpl<System, Event> {
    prologue: Vec<Event>,
    events: Vec<Event>,
    epilogue: Vec<Event>,
    resolve: Box<dyn Fn(Event, System) -> System>,
}

impl<System, Event: Clone> KnowledgeEngine<System, Event, System>
    for KnowledgeEngineImpl<System, Event>
{
    fn queue_event(self, event: Event) -> Self {
        let mut new = self;
        new.events.push(event);
        new
    }

    fn queue_prologue(self, event: Event) -> Self {
        let mut new = self;
        new.prologue.push(event);
        new
    }

    fn queue_epilogue(self, event: Event) -> Self {
        let mut new = self;
        new.epilogue.push(event);
        new
    }

    fn resolve_with<
        F: FnOnce(&mut dyn Iterator<Item = Event>) -> &mut dyn Iterator<Item = Event>,
        G: Fn() -> System,
    >(
        self,
        init: G,
        comb: F,
    ) -> System {
        let mut system = init();
        for event in self.prologue.into_iter() {
            system = (self.resolve)(event, system);
        }

        let mut events_iter = self.events.into_iter();
        for event in comb(&mut events_iter) {
            system = (self.resolve)(event, system);
        }

        for event in self.epilogue.into_iter() {
            system = (self.resolve)(event, system);
        }
        system
    }
}