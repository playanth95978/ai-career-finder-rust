import dayjs from 'dayjs/esm';

import { IOfferPositioning, NewOfferPositioning } from './offer-positioning.model';

export const sampleWithRequiredData: IOfferPositioning = {
  id: 1938,
  userId: 'immense',
  result: '../fake-data/blob/hipster.txt',
};

export const sampleWithPartialData: IOfferPositioning = {
  id: 1662,
  userId: 'financer quoique passablement',
  result: '../fake-data/blob/hipster.txt',
  createdAt: dayjs('2023-12-11T01:10'),
};

export const sampleWithFullData: IOfferPositioning = {
  id: 3294,
  userId: 'instituer grossir circulaire',
  result: '../fake-data/blob/hipster.txt',
  createdAt: dayjs('2023-12-10T18:25'),
};

export const sampleWithNewData: NewOfferPositioning = {
  userId: 'probablement',
  result: '../fake-data/blob/hipster.txt',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
