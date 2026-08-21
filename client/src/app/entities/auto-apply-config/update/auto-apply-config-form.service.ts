import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import { IAutoApplyConfig, NewAutoApplyConfig } from '../auto-apply-config.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IAutoApplyConfig for edit and NewAutoApplyConfigFormGroupInput for create.
 */
type AutoApplyConfigFormGroupInput = IAutoApplyConfig | PartialWithRequiredKeyOf<NewAutoApplyConfig>;

type AutoApplyConfigFormDefaults = Pick<NewAutoApplyConfig, 'id'>;

type AutoApplyConfigFormGroupContent = {
  id: FormControl<IAutoApplyConfig['id'] | NewAutoApplyConfig['id']>;
  userId: FormControl<IAutoApplyConfig['userId']>;
  mode: FormControl<IAutoApplyConfig['mode']>;
  minScore: FormControl<IAutoApplyConfig['minScore']>;
  maxPerDay: FormControl<IAutoApplyConfig['maxPerDay']>;
  sources: FormControl<IAutoApplyConfig['sources']>;
};

export type AutoApplyConfigFormGroup = FormGroup<AutoApplyConfigFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class AutoApplyConfigFormService {
  createAutoApplyConfigFormGroup(autoApplyConfig?: AutoApplyConfigFormGroupInput): AutoApplyConfigFormGroup {
    const autoApplyConfigRawValue = {
      ...this.getFormDefaults(),
      ...(autoApplyConfig ?? { id: null }),
    };

    return new FormGroup<AutoApplyConfigFormGroupContent>({
      id: new FormControl(
        { value: autoApplyConfigRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(autoApplyConfigRawValue.userId, {
        validators: [Validators.required],
      }),
      mode: new FormControl(autoApplyConfigRawValue.mode),
      minScore: new FormControl(autoApplyConfigRawValue.minScore),
      maxPerDay: new FormControl(autoApplyConfigRawValue.maxPerDay),
      sources: new FormControl(autoApplyConfigRawValue.sources),
    });
  }

  getAutoApplyConfig(form: AutoApplyConfigFormGroup): IAutoApplyConfig | NewAutoApplyConfig {
    return form.getRawValue();
  }

  resetForm(form: AutoApplyConfigFormGroup, autoApplyConfig: AutoApplyConfigFormGroupInput): void {
    const autoApplyConfigRawValue = { ...this.getFormDefaults(), ...autoApplyConfig };
    form.reset({
      ...autoApplyConfigRawValue,
      id: { value: autoApplyConfigRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): AutoApplyConfigFormDefaults {
    return {
      id: null,
    };
  }
}
