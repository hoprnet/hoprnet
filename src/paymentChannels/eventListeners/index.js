module.exports = self => ({
    closingListener: require('./close')(self),
    openingListener: require('./open')(self),
    openedForListener: require('./openedFor')(self)
})