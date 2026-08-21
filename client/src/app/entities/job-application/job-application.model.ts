import dayjs from 'dayjs/esm';

import { ICandidateProfile } from 'app/entities/candidate-profile/candidate-profile.model';
import { ApplicationStatus } from 'app/entities/enumerations/application-status.model';
import { IJobOffer } from 'app/entities/job-offer/job-offer.model';

export interface IJobApplication {
  id: number;
  userId?: string | null;
  status?: keyof typeof ApplicationStatus | null;
  coverLetter?: string | null;
  notes?: string | null;
  matchScore?: number | null;
  createdAt?: dayjs.Dayjs | null;
  updatedAt?: dayjs.Dayjs | null;
  appliedAt?: dayjs.Dayjs | null;
  jobOffer?: IJobOffer | null;
  candidateProfile?: ICandidateProfile | null;
}

export type NewJobApplication = Omit<IJobApplication, 'id'> & { id: null };
