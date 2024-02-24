import axios, { AxiosInstance } from 'axios';

import { SocksProxyAgent } from 'socks-proxy-agent';
import { URL } from 'url';

import { LnrpcAddInvoiceResponse, LnrpcInvoice } from './types';
import logger from '../logger';

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

  async lightningAddInvoice(createInvoiceArgs: LnrpcInvoice): Promise<any> {
    console.log('Making address');

    // const resp = await this.axios.post<LnrpcAddInvoiceResponse>(`v1/invoices`, createInvoiceArgs);
    // const invoice = resp.data;
    // return invoice;
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
