import dayjs from 'dayjs/esm';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';

export interface IOfferTailoredResume {
  id: number;
  userId?: string | null;
  data?: string | null;
  title?: string | null;
  createdAt?: dayjs.Dayjs | null;
  jobOffer?: IJobOffer | null;
}

export type NewOfferTailoredResume = Omit<IOfferTailoredResume, 'id'> & { id: null };
