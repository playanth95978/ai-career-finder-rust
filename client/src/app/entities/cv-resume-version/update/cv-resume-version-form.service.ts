import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { ICvResumeVersion, NewCvResumeVersion } from '../cv-resume-version.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts ICvResumeVersion for edit and NewCvResumeVersionFormGroupInput for create.
 */
type CvResumeVersionFormGroupInput = ICvResumeVersion | PartialWithRequiredKeyOf<NewCvResumeVersion>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends ICvResumeVersion | NewCvResumeVersion> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

type CvResumeVersionFormRawValue = FormValueOf<ICvResumeVersion>;

type NewCvResumeVersionFormRawValue = FormValueOf<NewCvResumeVersion>;

type CvResumeVersionFormDefaults = Pick<NewCvResumeVersion, 'id' | 'createdAt'>;

type CvResumeVersionFormGroupContent = {
  id: FormControl<CvResumeVersionFormRawValue['id'] | NewCvResumeVersion['id']>;
  versionNumber: FormControl<CvResumeVersionFormRawValue['versionNumber']>;
  title: FormControl<CvResumeVersionFormRawValue['title']>;
  template: FormControl<CvResumeVersionFormRawValue['template']>;
  data: FormControl<CvResumeVersionFormRawValue['data']>;
  createdAt: FormControl<CvResumeVersionFormRawValue['createdAt']>;
  resume: FormControl<CvResumeVersionFormRawValue['resume']>;
};

export type CvResumeVersionFormGroup = FormGroup<CvResumeVersionFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class CvResumeVersionFormService {
  createCvResumeVersionFormGroup(cvResumeVersion?: CvResumeVersionFormGroupInput): CvResumeVersionFormGroup {
    const cvResumeVersionRawValue = this.convertCvResumeVersionToCvResumeVersionRawValue({
      ...this.getFormDefaults(),
      ...(cvResumeVersion ?? { id: null }),
    });

    return new FormGroup<CvResumeVersionFormGroupContent>({
      id: new FormControl(
        { value: cvResumeVersionRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      versionNumber: new FormControl(cvResumeVersionRawValue.versionNumber, {
        validators: [Validators.required],
      }),
      title: new FormControl(cvResumeVersionRawValue.title),
      template: new FormControl(cvResumeVersionRawValue.template),
      data: new FormControl(cvResumeVersionRawValue.data, {
        validators: [Validators.required],
      }),
      createdAt: new FormControl(cvResumeVersionRawValue.createdAt),
      resume: new FormControl(cvResumeVersionRawValue.resume),
    });
  }

  getCvResumeVersion(form: CvResumeVersionFormGroup): ICvResumeVersion | NewCvResumeVersion {
    return this.convertCvResumeVersionRawValueToCvResumeVersion(form.getRawValue());
  }

  resetForm(form: CvResumeVersionFormGroup, cvResumeVersion: CvResumeVersionFormGroupInput): void {
    const cvResumeVersionRawValue = this.convertCvResumeVersionToCvResumeVersionRawValue({ ...this.getFormDefaults(), ...cvResumeVersion });
    form.reset({
      ...cvResumeVersionRawValue,
      id: { value: cvResumeVersionRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): CvResumeVersionFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
    };
  }

  private convertCvResumeVersionRawValueToCvResumeVersion(
    rawCvResumeVersion: CvResumeVersionFormRawValue | NewCvResumeVersionFormRawValue,
  ): ICvResumeVersion | NewCvResumeVersion {
    return {
      ...rawCvResumeVersion,
      createdAt: dayjs(rawCvResumeVersion.createdAt, DATE_TIME_FORMAT),
    };
  }

  private convertCvResumeVersionToCvResumeVersionRawValue(
    cvResumeVersion: ICvResumeVersion | (Partial<NewCvResumeVersion> & CvResumeVersionFormDefaults),
  ): CvResumeVersionFormRawValue | PartialWithRequiredKeyOf<NewCvResumeVersionFormRawValue> {
    return {
      ...cvResumeVersion,
      createdAt: cvResumeVersion.createdAt ? cvResumeVersion.createdAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
