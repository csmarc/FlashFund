import { engine } from 'express-handlebars';
import express from 'express';
import helmet from 'helmet';
import morgan from 'morgan';

import { wellknown } from './routes/well-known';
import { health } from './routes/health';
import bodyParser from 'body-parser';
import logger from './shared/logger';
import db from './shared/db';

//randy
var cookieParser = require('cookie-parser');
var cookieSession = require('cookie-session');
const { promisify } = require('util');
const exec = promisify(require('child_process').exec);

const PORT = process.env.PORT || 3000;
const NODE_ENV = process.env.NODE_ENV || 'development';

const app = express();

// Security Middleware
app.use(helmet());
app.use(bodyParser.json());

app.use(
  cookieSession({
    name: 'session',
    secret: 'secret',

    // Cookie Options
    maxAge: 24 * 60 * 60 * 1000 // 24 hours
  })
);

// Remove Express X-Powered-By headers
app.disable('x-powered-by');

app.engine('handlebars', engine());
app.set('view engine', 'handlebars');
app.set('views', __dirname + '/views');

// Logger
app.use(
  morgan('common', {
    stream: {
      write: (message) => logger.info(message.trim())
    }
  })
);

// Error
app.on('error', (err) => {
  logger.error('server error', err);
});

// Welcome Screen
app.get('/', function (req, res) {
  res.render('index', {
    domain: process.env.LNADDR_DOMAIN
  });
});

// Welcome Screen
app.get('/test', function (req, res) {
  res.render('test', {
    domain: process.env.LNADDR_DOMAIN
  });
});

const fse = require('fs-extra');

app.post('/post', async function (req, res) {
  console.log('BODY', req.body, '\n\n\n');
  let { body } = req;

  ['node_id', 'device_crt', 'device_key'].forEach((v) => {
    if (!body[v]) {
      return res.status(400).send(`The request must contain a value for ${v}`);
    }
  });

  let { lastInsertRowid } = db
    .prepare(
      'INSERT INTO user (node_id, device_crt, device_key) VALUES (@node_id, @device_crt, @device_key);'
    )
    .run(body);

  await fse.outputFile(`user-certs/${body.node_id}/device.crt`, body.device_crt);
  await fse.outputFile(`user-certs/${body.node_id}/device-key.pem`, body.device_key);

  let users = db.prepare('SELECT * FROM user;').all();
  console.log(users);

  res.status(200).send();
});

const secp256k1 = require('secp256k1');

app.post('/login', async function (req, res) {
  console.log('BODY', req.body, '\n\n\n');
  let { body } = req;

  ['signer_pubkey', 'signer_msg', 'signer_sig'].forEach((v) => {
    if (!body[v]) {
      return res.status(400).send(`The request must contain a value for ${v}`);
    }
  });

  const { createHash } = require('crypto');
  let hash0 = createHash('sha256').update(`Lightning Signed Message:${body.signer_msg}`).digest();
  let hash1 = createHash('sha256').update(hash0).digest();

  let pkey = Buffer.from(body.signer_pubkey, 'hex');

  let sig = Buffer.from(body.signer_sig, 'hex');

  if (secp256k1.ecdsaVerify(sig, hash1, pkey)) {
    let { session } = req as any;
    session.pubkey = body.signer_pubkey;
    console.log(session.pubkey);
  }

  res.status(200).send();
});

app.get('/invoices', async function (req, res) {
  console.log((req as any).session.pubkey, '\n\n\n');
  let { session } = req as any;
  if (!session.pubkey) {
    return res.status(400).send('No active session found');
  }

  /*
    Make terminal call to rust code
  */

  let dc = `${__dirname}/../user-certs/${session.pubkey}/device.crt`;
  let dk = `${__dirname}/../user-certs/${session.pubkey}/device-key.pem`;

  let xc = `${__dirname}/gl/target/debug/gl getinvoices ${session.pubkey} ${dc} ${dk}`;
  console.log(xc);
  const lsOut = await exec(xc);
  console.log(lsOut.stdout);

  //Parse the stdout into invoice array

  //return invoice array
  res.json(JSON.parse(lsOut.stdout));
});

// Health Route
app.use('/healthz', health);

// Lightning Address
app.use('/.well-known', wellknown);

app.get('/:file', (req, res) => {
  res.sendFile(`${__dirname}/views/${req.params.file}`);
});

// Start Server
app.listen(PORT, () => {
  logger.info(`Lightning Address Server listening on port ${PORT} in ${NODE_ENV} mode`);
});
