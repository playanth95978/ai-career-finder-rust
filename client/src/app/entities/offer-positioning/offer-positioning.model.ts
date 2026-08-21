import dayjs from 'dayjs/esm';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';

export interface IOfferPositioning {
  id: number;
  userId?: string | null;
  result?: string | null;
  createdAt?: dayjs.Dayjs | null;
  jobOffer?: IJobOffer | null;
}

export type NewOfferPositioning = Omit<IOfferPositioning, 'id'> & { id: null };
