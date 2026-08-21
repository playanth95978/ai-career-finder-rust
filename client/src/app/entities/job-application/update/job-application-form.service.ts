import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IJobApplication, NewJobApplication } from '../job-application.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IJobApplication for edit and NewJobApplicationFormGroupInput for create.
 */
type JobApplicationFormGroupInput = IJobApplication | PartialWithRequiredKeyOf<NewJobApplication>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IJobApplication | NewJobApplication> = Omit<T, 'createdAt' | 'updatedAt' | 'appliedAt'> & {
  createdAt?: string | null;
  updatedAt?: string | null;
  appliedAt?: string | null;
};

type JobApplicationFormRawValue = FormValueOf<IJobApplication>;

type NewJobApplicationFormRawValue = FormValueOf<NewJobApplication>;

type JobApplicationFormDefaults = Pick<NewJobApplication, 'id' | 'createdAt' | 'updatedAt' | 'appliedAt'>;

type JobApplicationFormGroupContent = {
  id: FormControl<JobApplicationFormRawValue['id'] | NewJobApplication['id']>;
  userId: FormControl<JobApplicationFormRawValue['userId']>;
  status: FormControl<JobApplicationFormRawValue['status']>;
  coverLetter: FormControl<JobApplicationFormRawValue['coverLetter']>;
  notes: FormControl<JobApplicationFormRawValue['notes']>;
  matchScore: FormControl<JobApplicationFormRawValue['matchScore']>;
  createdAt: FormControl<JobApplicationFormRawValue['createdAt']>;
  updatedAt: FormControl<JobApplicationFormRawValue['updatedAt']>;
  appliedAt: FormControl<JobApplicationFormRawValue['appliedAt']>;
  jobOffer: FormControl<JobApplicationFormRawValue['jobOffer']>;
  candidateProfile: FormControl<JobApplicationFormRawValue['candidateProfile']>;
};

export type JobApplicationFormGroup = FormGroup<JobApplicationFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class JobApplicationFormService {
  createJobApplicationFormGroup(jobApplication?: JobApplicationFormGroupInput): JobApplicationFormGroup {
    const jobApplicationRawValue = this.convertJobApplicationToJobApplicationRawValue({
      ...this.getFormDefaults(),
      ...(jobApplication ?? { id: null }),
    });

    return new FormGroup<JobApplicationFormGroupContent>({
      id: new FormControl(
        { value: jobApplicationRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(jobApplicationRawValue.userId, {
        validators: [Validators.required],
      }),
      status: new FormControl(jobApplicationRawValue.status),
      coverLetter: new FormControl(jobApplicationRawValue.coverLetter),
      notes: new FormControl(jobApplicationRawValue.notes),
      matchScore: new FormControl(jobApplicationRawValue.matchScore),
      createdAt: new FormControl(jobApplicationRawValue.createdAt),
      updatedAt: new FormControl(jobApplicationRawValue.updatedAt),
      appliedAt: new FormControl(jobApplicationRawValue.appliedAt),
      jobOffer: new FormControl(jobApplicationRawValue.jobOffer),
      candidateProfile: new FormControl(jobApplicationRawValue.candidateProfile),
    });
  }

  getJobApplication(form: JobApplicationFormGroup): IJobApplication | NewJobApplication {
    return this.convertJobApplicationRawValueToJobApplication(form.getRawValue());
  }

  resetForm(form: JobApplicationFormGroup, jobApplication: JobApplicationFormGroupInput): void {
    const jobApplicationRawValue = this.convertJobApplicationToJobApplicationRawValue({ ...this.getFormDefaults(), ...jobApplication });
    form.reset({
      ...jobApplicationRawValue,
      id: { value: jobApplicationRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): JobApplicationFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
      updatedAt: currentTime,
      appliedAt: currentTime,
    };
  }

  private convertJobApplicationRawValueToJobApplication(
    rawJobApplication: JobApplicationFormRawValue | NewJobApplicationFormRawValue,
  ): IJobApplication | NewJobApplication {
    return {
      ...rawJobApplication,
      createdAt: dayjs(rawJobApplication.createdAt, DATE_TIME_FORMAT),
      updatedAt: dayjs(rawJobApplication.updatedAt, DATE_TIME_FORMAT),
      appliedAt: dayjs(rawJobApplication.appliedAt, DATE_TIME_FORMAT),
    };
  }

  private convertJobApplicationToJobApplicationRawValue(
    jobApplication: IJobApplication | (Partial<NewJobApplication> & JobApplicationFormDefaults),
  ): JobApplicationFormRawValue | PartialWithRequiredKeyOf<NewJobApplicationFormRawValue> {
    return {
      ...jobApplication,
      createdAt: jobApplication.createdAt ? jobApplication.createdAt.format(DATE_TIME_FORMAT) : undefined,
      updatedAt: jobApplication.updatedAt ? jobApplication.updatedAt.format(DATE_TIME_FORMAT) : undefined,
      appliedAt: jobApplication.appliedAt ? jobApplication.appliedAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
