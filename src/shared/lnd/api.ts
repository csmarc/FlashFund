import axios, { AxiosInstance } from 'axios';

import { SocksProxyAgent } from 'socks-proxy-agent';
import { URL } from 'url';

import { LnrpcAddInvoiceResponse, LnrpcInvoice } from './types';
import logger from '../logger';
import { parse } from 'path';
const { promisify } = require('util');
const exec = promisify(require('child_process').exec);

const BASE_URL = process.env.LNADDR_CORE_LIGHTNING;
// const MACAROON = process.env.LNADDR_LND_REST_MACAROON_HEX;
// const TOR_PROXY_URL = process.env.LNADDR_TOR_PROXY_URL || 'socks5h://localhost:9050';
const WEBHOOK_URL = process.env.LNADDR_NOTIFICATION_WEBHOOK;

if (!BASE_URL) {
  throw new Error('Misconfigured Environment Variables');
}

interface LightningAPIArgs {
  baseUrl: string;
}

class LightningAPI {
  baseUrl: string;
  axios: AxiosInstance;

  constructor(args: LightningAPIArgs) {
    this.baseUrl = args.baseUrl;
    this.axios = axios.create({
      baseURL: args.baseUrl.endsWith('/') ? args.baseUrl : `${args.baseUrl}/`
    });
  }

  async lightningAddInvoice(createInvoiceArgs: LnrpcInvoice, session: any): Promise<any> {
    /*
      Make terminal call to rust code
    */

    let dc = `${__dirname}/../../../user-certs/${session.pubkey}/device.crt`;
    let dk = `${__dirname}/../../../user-certs/${session.pubkey}/device-key.pem`;

    let xc = `/Users/mauricepoirrierchuden/repo/opensource/FlashFund/src/gl/target/debug/gl createinvoice ${session.pubkey} ${dc} ${dk}`;
    console.log(xc);

    console.log('getting invoice');
    const lsOut = await exec(xc);
    console.log(lsOut.stdout);

    //return invoice array
    return JSON.parse(lsOut.stdout);
  }

  async sendWebhookNotification(data: any) {
    if (!WEBHOOK_URL) {
      logger.debug('Not sending Notification. LNADDR_NOTIFICATION_WEBHOOK not set');
    } else {
      logger.debug('Sending Webhook Notification', { url: WEBHOOK_URL, data });
      try {
        await axios.post(WEBHOOK_URL, data, {});
      } catch (error) {
        logger.error('Error sending Webhook Notification', error);
      }
    }
  }
}

export const lightningApi = new LightningAPI({
  baseUrl: BASE_URL
});
