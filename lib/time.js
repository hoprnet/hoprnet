"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.durations = {
    seconds(seconds) {
        return seconds * 1e3;
    },
    minutes(minutes) {
        return minutes * exports.durations.seconds(60);
    },
    hours(hours) {
        return hours * exports.durations.minutes(60);
    },
    days(days) {
        return days * exports.durations.hours(24);
    },
};
