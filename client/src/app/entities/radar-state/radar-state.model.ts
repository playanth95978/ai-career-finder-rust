import dayjs from 'dayjs/esm';

export interface IRadarState {
  id: number;
  userId?: string | null;
  lastOfferAt?: dayjs.Dayjs | null;
}

export type NewRadarState = Omit<IRadarState, 'id'> & { id: null };
