import dayjs from 'dayjs/esm';

export interface ICandidateProfile {
  id: number;
  userId?: string | null;
  fullName?: string | null;
  email?: string | null;
  location?: string | null;
  yearsOfExperience?: number | null;
  skills?: string | null;
  experiences?: string | null;
  preferredRoles?: string | null;
  languages?: string | null;
  education?: string | null;
  certifications?: string | null;
  rawMarkdown?: string | null;
  cvFilename?: string | null;
  embeddingModel?: string | null;
  embeddedAt?: dayjs.Dayjs | null;
  createdAt?: dayjs.Dayjs | null;
  updatedAt?: dayjs.Dayjs | null;
}

export type NewCandidateProfile = Omit<ICandidateProfile, 'id'> & { id: null };
