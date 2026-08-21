import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { ICandidateProfile, NewCandidateProfile } from '../candidate-profile.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts ICandidateProfile for edit and NewCandidateProfileFormGroupInput for create.
 */
type CandidateProfileFormGroupInput = ICandidateProfile | PartialWithRequiredKeyOf<NewCandidateProfile>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends ICandidateProfile | NewCandidateProfile> = Omit<T, 'embeddedAt' | 'createdAt' | 'updatedAt'> & {
  embeddedAt?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
};

type CandidateProfileFormRawValue = FormValueOf<ICandidateProfile>;

type NewCandidateProfileFormRawValue = FormValueOf<NewCandidateProfile>;

type CandidateProfileFormDefaults = Pick<NewCandidateProfile, 'id' | 'embeddedAt' | 'createdAt' | 'updatedAt'>;

type CandidateProfileFormGroupContent = {
  id: FormControl<CandidateProfileFormRawValue['id'] | NewCandidateProfile['id']>;
  userId: FormControl<CandidateProfileFormRawValue['userId']>;
  fullName: FormControl<CandidateProfileFormRawValue['fullName']>;
  email: FormControl<CandidateProfileFormRawValue['email']>;
  location: FormControl<CandidateProfileFormRawValue['location']>;
  yearsOfExperience: FormControl<CandidateProfileFormRawValue['yearsOfExperience']>;
  skills: FormControl<CandidateProfileFormRawValue['skills']>;
  experiences: FormControl<CandidateProfileFormRawValue['experiences']>;
  preferredRoles: FormControl<CandidateProfileFormRawValue['preferredRoles']>;
  languages: FormControl<CandidateProfileFormRawValue['languages']>;
  education: FormControl<CandidateProfileFormRawValue['education']>;
  certifications: FormControl<CandidateProfileFormRawValue['certifications']>;
  rawMarkdown: FormControl<CandidateProfileFormRawValue['rawMarkdown']>;
  cvFilename: FormControl<CandidateProfileFormRawValue['cvFilename']>;
  embeddingModel: FormControl<CandidateProfileFormRawValue['embeddingModel']>;
  embeddedAt: FormControl<CandidateProfileFormRawValue['embeddedAt']>;
  createdAt: FormControl<CandidateProfileFormRawValue['createdAt']>;
  updatedAt: FormControl<CandidateProfileFormRawValue['updatedAt']>;
};

export type CandidateProfileFormGroup = FormGroup<CandidateProfileFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class CandidateProfileFormService {
  createCandidateProfileFormGroup(candidateProfile?: CandidateProfileFormGroupInput): CandidateProfileFormGroup {
    const candidateProfileRawValue = this.convertCandidateProfileToCandidateProfileRawValue({
      ...this.getFormDefaults(),
      ...(candidateProfile ?? { id: null }),
    });

    return new FormGroup<CandidateProfileFormGroupContent>({
      id: new FormControl(
        { value: candidateProfileRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(candidateProfileRawValue.userId, {
        validators: [Validators.required],
      }),
      fullName: new FormControl(candidateProfileRawValue.fullName),
      email: new FormControl(candidateProfileRawValue.email),
      location: new FormControl(candidateProfileRawValue.location),
      yearsOfExperience: new FormControl(candidateProfileRawValue.yearsOfExperience),
      skills: new FormControl(candidateProfileRawValue.skills),
      experiences: new FormControl(candidateProfileRawValue.experiences),
      preferredRoles: new FormControl(candidateProfileRawValue.preferredRoles),
      languages: new FormControl(candidateProfileRawValue.languages),
      education: new FormControl(candidateProfileRawValue.education),
      certifications: new FormControl(candidateProfileRawValue.certifications),
      rawMarkdown: new FormControl(candidateProfileRawValue.rawMarkdown),
      cvFilename: new FormControl(candidateProfileRawValue.cvFilename),
      embeddingModel: new FormControl(candidateProfileRawValue.embeddingModel),
      embeddedAt: new FormControl(candidateProfileRawValue.embeddedAt),
      createdAt: new FormControl(candidateProfileRawValue.createdAt),
      updatedAt: new FormControl(candidateProfileRawValue.updatedAt),
    });
  }

  getCandidateProfile(form: CandidateProfileFormGroup): ICandidateProfile | NewCandidateProfile {
    return this.convertCandidateProfileRawValueToCandidateProfile(form.getRawValue());
  }

  resetForm(form: CandidateProfileFormGroup, candidateProfile: CandidateProfileFormGroupInput): void {
    const candidateProfileRawValue = this.convertCandidateProfileToCandidateProfileRawValue({
      ...this.getFormDefaults(),
      ...candidateProfile,
    });
    form.reset({
      ...candidateProfileRawValue,
      id: { value: candidateProfileRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): CandidateProfileFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      embeddedAt: currentTime,
      createdAt: currentTime,
      updatedAt: currentTime,
    };
  }

  private convertCandidateProfileRawValueToCandidateProfile(
    rawCandidateProfile: CandidateProfileFormRawValue | NewCandidateProfileFormRawValue,
  ): ICandidateProfile | NewCandidateProfile {
    return {
      ...rawCandidateProfile,
      embeddedAt: dayjs(rawCandidateProfile.embeddedAt, DATE_TIME_FORMAT),
      createdAt: dayjs(rawCandidateProfile.createdAt, DATE_TIME_FORMAT),
      updatedAt: dayjs(rawCandidateProfile.updatedAt, DATE_TIME_FORMAT),
    };
  }

  private convertCandidateProfileToCandidateProfileRawValue(
    candidateProfile: ICandidateProfile | (Partial<NewCandidateProfile> & CandidateProfileFormDefaults),
  ): CandidateProfileFormRawValue | PartialWithRequiredKeyOf<NewCandidateProfileFormRawValue> {
    return {
      ...candidateProfile,
      embeddedAt: candidateProfile.embeddedAt ? candidateProfile.embeddedAt.format(DATE_TIME_FORMAT) : undefined,
      createdAt: candidateProfile.createdAt ? candidateProfile.createdAt.format(DATE_TIME_FORMAT) : undefined,
      updatedAt: candidateProfile.updatedAt ? candidateProfile.updatedAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
