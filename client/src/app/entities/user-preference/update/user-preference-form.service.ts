import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import { IUserPreference, NewUserPreference } from '../user-preference.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IUserPreference for edit and NewUserPreferenceFormGroupInput for create.
 */
type UserPreferenceFormGroupInput = IUserPreference | PartialWithRequiredKeyOf<NewUserPreference>;

type UserPreferenceFormDefaults = Pick<NewUserPreference, 'id' | 'remoteOnly'>;

type UserPreferenceFormGroupContent = {
  id: FormControl<IUserPreference['id'] | NewUserPreference['id']>;
  userId: FormControl<IUserPreference['userId']>;
  remoteOnly: FormControl<IUserPreference['remoteOnly']>;
  contractType: FormControl<IUserPreference['contractType']>;
  salaryMin: FormControl<IUserPreference['salaryMin']>;
  salaryMax: FormControl<IUserPreference['salaryMax']>;
  preferredRoles: FormControl<IUserPreference['preferredRoles']>;
  excludedTechnologies: FormControl<IUserPreference['excludedTechnologies']>;
  preferredLocations: FormControl<IUserPreference['preferredLocations']>;
};

export type UserPreferenceFormGroup = FormGroup<UserPreferenceFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class UserPreferenceFormService {
  createUserPreferenceFormGroup(userPreference?: UserPreferenceFormGroupInput): UserPreferenceFormGroup {
    const userPreferenceRawValue = {
      ...this.getFormDefaults(),
      ...(userPreference ?? { id: null }),
    };

    return new FormGroup<UserPreferenceFormGroupContent>({
      id: new FormControl(
        { value: userPreferenceRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(userPreferenceRawValue.userId, {
        validators: [Validators.required],
      }),
      remoteOnly: new FormControl(userPreferenceRawValue.remoteOnly),
      contractType: new FormControl(userPreferenceRawValue.contractType),
      salaryMin: new FormControl(userPreferenceRawValue.salaryMin),
      salaryMax: new FormControl(userPreferenceRawValue.salaryMax),
      preferredRoles: new FormControl(userPreferenceRawValue.preferredRoles),
      excludedTechnologies: new FormControl(userPreferenceRawValue.excludedTechnologies),
      preferredLocations: new FormControl(userPreferenceRawValue.preferredLocations),
    });
  }

  getUserPreference(form: UserPreferenceFormGroup): IUserPreference | NewUserPreference {
    return form.getRawValue();
  }

  resetForm(form: UserPreferenceFormGroup, userPreference: UserPreferenceFormGroupInput): void {
    const userPreferenceRawValue = { ...this.getFormDefaults(), ...userPreference };
    form.reset({
      ...userPreferenceRawValue,
      id: { value: userPreferenceRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): UserPreferenceFormDefaults {
    return {
      id: null,
      remoteOnly: false,
    };
  }
}
