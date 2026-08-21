import dayjs from 'dayjs/esm';

import { IRadarState, NewRadarState } from './radar-state.model';

export const sampleWithRequiredData: IRadarState = {
  id: 20706,
  userId: 'renseigner',
};

export const sampleWithPartialData: IRadarState = {
  id: 14574,
  userId: 'tant',
  lastOfferAt: dayjs('2023-12-11T05:49'),
};

export const sampleWithFullData: IRadarState = {
  id: 25932,
  userId: 'pendant que du côté de',
  lastOfferAt: dayjs('2023-12-10T21:15'),
};

export const sampleWithNewData: NewRadarState = {
  userId: 'turquoise',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
