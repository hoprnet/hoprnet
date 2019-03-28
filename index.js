const serve = require('koa-static');
const Koa = require('koa');
const app = new Koa();

app.use(serve('assets'));

const port = 3000;

app.listen(port);

console.log('listening on port ' + port);