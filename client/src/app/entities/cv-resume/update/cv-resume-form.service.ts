import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { ICvResume, NewCvResume } from '../cv-resume.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts ICvResume for edit and NewCvResumeFormGroupInput for create.
 */
type CvResumeFormGroupInput = ICvResume | PartialWithRequiredKeyOf<NewCvResume>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends ICvResume | NewCvResume> = Omit<T, 'createdAt' | 'updatedAt'> & {
  createdAt?: string | null;
  updatedAt?: string | null;
};

type CvResumeFormRawValue = FormValueOf<ICvResume>;

type NewCvResumeFormRawValue = FormValueOf<NewCvResume>;

type CvResumeFormDefaults = Pick<NewCvResume, 'id' | 'createdAt' | 'updatedAt'>;

type CvResumeFormGroupContent = {
  id: FormControl<CvResumeFormRawValue['id'] | NewCvResume['id']>;
  userId: FormControl<CvResumeFormRawValue['userId']>;
  title: FormControl<CvResumeFormRawValue['title']>;
  template: FormControl<CvResumeFormRawValue['template']>;
  data: FormControl<CvResumeFormRawValue['data']>;
  versionNumber: FormControl<CvResumeFormRawValue['versionNumber']>;
  createdAt: FormControl<CvResumeFormRawValue['createdAt']>;
  updatedAt: FormControl<CvResumeFormRawValue['updatedAt']>;
};

export type CvResumeFormGroup = FormGroup<CvResumeFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class CvResumeFormService {
  createCvResumeFormGroup(cvResume?: CvResumeFormGroupInput): CvResumeFormGroup {
    const cvResumeRawValue = this.convertCvResumeToCvResumeRawValue({
      ...this.getFormDefaults(),
      ...(cvResume ?? { id: null }),
    });

    return new FormGroup<CvResumeFormGroupContent>({
      id: new FormControl(
        { value: cvResumeRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(cvResumeRawValue.userId, {
        validators: [Validators.required],
      }),
      title: new FormControl(cvResumeRawValue.title),
      template: new FormControl(cvResumeRawValue.template),
      data: new FormControl(cvResumeRawValue.data, {
        validators: [Validators.required],
      }),
      versionNumber: new FormControl(cvResumeRawValue.versionNumber, {
        validators: [Validators.required],
      }),
      createdAt: new FormControl(cvResumeRawValue.createdAt),
      updatedAt: new FormControl(cvResumeRawValue.updatedAt),
    });
  }

  getCvResume(form: CvResumeFormGroup): ICvResume | NewCvResume {
    return this.convertCvResumeRawValueToCvResume(form.getRawValue());
  }

  resetForm(form: CvResumeFormGroup, cvResume: CvResumeFormGroupInput): void {
    const cvResumeRawValue = this.convertCvResumeToCvResumeRawValue({ ...this.getFormDefaults(), ...cvResume });
    form.reset({
      ...cvResumeRawValue,
      id: { value: cvResumeRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): CvResumeFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
      updatedAt: currentTime,
    };
  }

  private convertCvResumeRawValueToCvResume(rawCvResume: CvResumeFormRawValue | NewCvResumeFormRawValue): ICvResume | NewCvResume {
    return {
      ...rawCvResume,
      createdAt: dayjs(rawCvResume.createdAt, DATE_TIME_FORMAT),
      updatedAt: dayjs(rawCvResume.updatedAt, DATE_TIME_FORMAT),
    };
  }

  private convertCvResumeToCvResumeRawValue(
    cvResume: ICvResume | (Partial<NewCvResume> & CvResumeFormDefaults),
  ): CvResumeFormRawValue | PartialWithRequiredKeyOf<NewCvResumeFormRawValue> {
    return {
      ...cvResume,
      createdAt: cvResume.createdAt ? cvResume.createdAt.format(DATE_TIME_FORMAT) : undefined,
      updatedAt: cvResume.updatedAt ? cvResume.updatedAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
