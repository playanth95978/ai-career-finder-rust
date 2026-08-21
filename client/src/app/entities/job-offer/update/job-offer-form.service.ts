import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IJobOffer, NewJobOffer } from '../job-offer.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IJobOffer for edit and NewJobOfferFormGroupInput for create.
 */
type JobOfferFormGroupInput = IJobOffer | PartialWithRequiredKeyOf<NewJobOffer>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IJobOffer | NewJobOffer> = Omit<
  T,
  'publishedAt' | 'createdAt' | 'indexedAt' | 'updatedAt' | 'expiresAt' | 'lastCheckedAt'
> & {
  publishedAt?: string | null;
  createdAt?: string | null;
  indexedAt?: string | null;
  updatedAt?: string | null;
  expiresAt?: string | null;
  lastCheckedAt?: string | null;
};

type JobOfferFormRawValue = FormValueOf<IJobOffer>;

type NewJobOfferFormRawValue = FormValueOf<NewJobOffer>;

type JobOfferFormDefaults = Pick<
  NewJobOffer,
  'id' | 'remote' | 'publishedAt' | 'createdAt' | 'indexedAt' | 'updatedAt' | 'expiresAt' | 'lastCheckedAt'
>;

type JobOfferFormGroupContent = {
  id: FormControl<JobOfferFormRawValue['id'] | NewJobOffer['id']>;
  title: FormControl<JobOfferFormRawValue['title']>;
  company: FormControl<JobOfferFormRawValue['company']>;
  location: FormControl<JobOfferFormRawValue['location']>;
  country: FormControl<JobOfferFormRawValue['country']>;
  remote: FormControl<JobOfferFormRawValue['remote']>;
  description: FormControl<JobOfferFormRawValue['description']>;
  searchText: FormControl<JobOfferFormRawValue['searchText']>;
  skills: FormControl<JobOfferFormRawValue['skills']>;
  metadata: FormControl<JobOfferFormRawValue['metadata']>;
  rawPayload: FormControl<JobOfferFormRawValue['rawPayload']>;
  contentHash: FormControl<JobOfferFormRawValue['contentHash']>;
  embeddingStatus: FormControl<JobOfferFormRawValue['embeddingStatus']>;
  embeddingModel: FormControl<JobOfferFormRawValue['embeddingModel']>;
  reindexVersion: FormControl<JobOfferFormRawValue['reindexVersion']>;
  retryCount: FormControl<JobOfferFormRawValue['retryCount']>;
  indexingError: FormControl<JobOfferFormRawValue['indexingError']>;
  source: FormControl<JobOfferFormRawValue['source']>;
  sourceId: FormControl<JobOfferFormRawValue['sourceId']>;
  applyUrl: FormControl<JobOfferFormRawValue['applyUrl']>;
  salaryMin: FormControl<JobOfferFormRawValue['salaryMin']>;
  salaryMax: FormControl<JobOfferFormRawValue['salaryMax']>;
  salaryCurrency: FormControl<JobOfferFormRawValue['salaryCurrency']>;
  contractType: FormControl<JobOfferFormRawValue['contractType']>;
  experienceLevel: FormControl<JobOfferFormRawValue['experienceLevel']>;
  category: FormControl<JobOfferFormRawValue['category']>;
  sourceCategory: FormControl<JobOfferFormRawValue['sourceCategory']>;
  publishedAt: FormControl<JobOfferFormRawValue['publishedAt']>;
  createdAt: FormControl<JobOfferFormRawValue['createdAt']>;
  indexedAt: FormControl<JobOfferFormRawValue['indexedAt']>;
  updatedAt: FormControl<JobOfferFormRawValue['updatedAt']>;
  expiresAt: FormControl<JobOfferFormRawValue['expiresAt']>;
  lastCheckedAt: FormControl<JobOfferFormRawValue['lastCheckedAt']>;
};

export type JobOfferFormGroup = FormGroup<JobOfferFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class JobOfferFormService {
  createJobOfferFormGroup(jobOffer?: JobOfferFormGroupInput): JobOfferFormGroup {
    const jobOfferRawValue = this.convertJobOfferToJobOfferRawValue({
      ...this.getFormDefaults(),
      ...(jobOffer ?? { id: null }),
    });

    return new FormGroup<JobOfferFormGroupContent>({
      id: new FormControl(
        { value: jobOfferRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      title: new FormControl(jobOfferRawValue.title, {
        validators: [Validators.required],
      }),
      company: new FormControl(jobOfferRawValue.company),
      location: new FormControl(jobOfferRawValue.location),
      country: new FormControl(jobOfferRawValue.country),
      remote: new FormControl(jobOfferRawValue.remote),
      description: new FormControl(jobOfferRawValue.description),
      searchText: new FormControl(jobOfferRawValue.searchText),
      skills: new FormControl(jobOfferRawValue.skills),
      metadata: new FormControl(jobOfferRawValue.metadata),
      rawPayload: new FormControl(jobOfferRawValue.rawPayload),
      contentHash: new FormControl(jobOfferRawValue.contentHash, {
        validators: [Validators.maxLength(64)],
      }),
      embeddingStatus: new FormControl(jobOfferRawValue.embeddingStatus),
      embeddingModel: new FormControl(jobOfferRawValue.embeddingModel),
      reindexVersion: new FormControl(jobOfferRawValue.reindexVersion),
      retryCount: new FormControl(jobOfferRawValue.retryCount),
      indexingError: new FormControl(jobOfferRawValue.indexingError),
      source: new FormControl(jobOfferRawValue.source),
      sourceId: new FormControl(jobOfferRawValue.sourceId),
      applyUrl: new FormControl(jobOfferRawValue.applyUrl),
      salaryMin: new FormControl(jobOfferRawValue.salaryMin),
      salaryMax: new FormControl(jobOfferRawValue.salaryMax),
      salaryCurrency: new FormControl(jobOfferRawValue.salaryCurrency),
      contractType: new FormControl(jobOfferRawValue.contractType),
      experienceLevel: new FormControl(jobOfferRawValue.experienceLevel),
      category: new FormControl(jobOfferRawValue.category),
      sourceCategory: new FormControl(jobOfferRawValue.sourceCategory),
      publishedAt: new FormControl(jobOfferRawValue.publishedAt),
      createdAt: new FormControl(jobOfferRawValue.createdAt),
      indexedAt: new FormControl(jobOfferRawValue.indexedAt),
      updatedAt: new FormControl(jobOfferRawValue.updatedAt),
      expiresAt: new FormControl(jobOfferRawValue.expiresAt),
      lastCheckedAt: new FormControl(jobOfferRawValue.lastCheckedAt),
    });
  }

  getJobOffer(form: JobOfferFormGroup): IJobOffer | NewJobOffer {
    return this.convertJobOfferRawValueToJobOffer(form.getRawValue());
  }

  resetForm(form: JobOfferFormGroup, jobOffer: JobOfferFormGroupInput): void {
    const jobOfferRawValue = this.convertJobOfferToJobOfferRawValue({ ...this.getFormDefaults(), ...jobOffer });
    form.reset({
      ...jobOfferRawValue,
      id: { value: jobOfferRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): JobOfferFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      remote: false,
      publishedAt: currentTime,
      createdAt: currentTime,
      indexedAt: currentTime,
      updatedAt: currentTime,
      expiresAt: currentTime,
      lastCheckedAt: currentTime,
    };
  }

  private convertJobOfferRawValueToJobOffer(rawJobOffer: JobOfferFormRawValue | NewJobOfferFormRawValue): IJobOffer | NewJobOffer {
    return {
      ...rawJobOffer,
      publishedAt: dayjs(rawJobOffer.publishedAt, DATE_TIME_FORMAT),
      createdAt: dayjs(rawJobOffer.createdAt, DATE_TIME_FORMAT),
      indexedAt: dayjs(rawJobOffer.indexedAt, DATE_TIME_FORMAT),
      updatedAt: dayjs(rawJobOffer.updatedAt, DATE_TIME_FORMAT),
      expiresAt: dayjs(rawJobOffer.expiresAt, DATE_TIME_FORMAT),
      lastCheckedAt: dayjs(rawJobOffer.lastCheckedAt, DATE_TIME_FORMAT),
    };
  }

  private convertJobOfferToJobOfferRawValue(
    jobOffer: IJobOffer | (Partial<NewJobOffer> & JobOfferFormDefaults),
  ): JobOfferFormRawValue | PartialWithRequiredKeyOf<NewJobOfferFormRawValue> {
    return {
      ...jobOffer,
      publishedAt: jobOffer.publishedAt ? jobOffer.publishedAt.format(DATE_TIME_FORMAT) : undefined,
      createdAt: jobOffer.createdAt ? jobOffer.createdAt.format(DATE_TIME_FORMAT) : undefined,
      indexedAt: jobOffer.indexedAt ? jobOffer.indexedAt.format(DATE_TIME_FORMAT) : undefined,
      updatedAt: jobOffer.updatedAt ? jobOffer.updatedAt.format(DATE_TIME_FORMAT) : undefined,
      expiresAt: jobOffer.expiresAt ? jobOffer.expiresAt.format(DATE_TIME_FORMAT) : undefined,
      lastCheckedAt: jobOffer.lastCheckedAt ? jobOffer.lastCheckedAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
