import dayjs from 'dayjs/esm';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';

export interface IRadarHit {
  id: number;
  userId?: string | null;
  score?: number | null;
  whyYou?: string | null;
  seen?: boolean | null;
  dismissed?: boolean | null;
  createdAt?: dayjs.Dayjs | null;
  jobOffer?: IJobOffer | null;
}

export type NewRadarHit = Omit<IRadarHit, 'id'> & { id: null };
