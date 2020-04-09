"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const store = new Map();
const addEvent = (name, event) => {
    const events = store.get(name) || [];
    events.push(event);
    store.set(name, events);
    return event;
};
exports.addEvent = addEvent;
const clearEvents = (name) => {
    const events = store.get(name) || [];
    // @TODO: needs testing
    for (const event of events) {
        event.removeAllListeners();
    }
    store.set(name, []);
};
exports.clearEvents = clearEvents;
const clearAllEvents = () => {
    for (const events of store.keys()) {
        clearEvents(events);
    }
};
exports.clearAllEvents = clearAllEvents;
