import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../user-preference.test-samples';

import { UserPreferenceFormService } from './user-preference-form.service';

describe('UserPreference Form Service', () => {
  let service: UserPreferenceFormService;

  beforeEach(() => {
    service = TestBed.inject(UserPreferenceFormService);
  });

  describe('Service methods', () => {
    describe('createUserPreferenceFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createUserPreferenceFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            remoteOnly: expect.any(Object),
            contractType: expect.any(Object),
            salaryMin: expect.any(Object),
            salaryMax: expect.any(Object),
            preferredRoles: expect.any(Object),
            excludedTechnologies: expect.any(Object),
            preferredLocations: expect.any(Object),
          }),
        );
      });

      it('passing IUserPreference should create a new form with FormGroup', () => {
        const formGroup = service.createUserPreferenceFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            remoteOnly: expect.any(Object),
            contractType: expect.any(Object),
            salaryMin: expect.any(Object),
            salaryMax: expect.any(Object),
            preferredRoles: expect.any(Object),
            excludedTechnologies: expect.any(Object),
            preferredLocations: expect.any(Object),
          }),
        );
      });
    });

    describe('getUserPreference', () => {
      it('should return NewUserPreference for default UserPreference initial value', () => {
        const formGroup = service.createUserPreferenceFormGroup(sampleWithNewData);

        const userPreference = service.getUserPreference(formGroup);

        expect(userPreference).toMatchObject(sampleWithNewData);
      });

      it('should return NewUserPreference for empty UserPreference initial value', () => {
        const formGroup = service.createUserPreferenceFormGroup();

        const userPreference = service.getUserPreference(formGroup);

        expect(userPreference).toMatchObject({});
      });

      it('should return IUserPreference', () => {
        const formGroup = service.createUserPreferenceFormGroup(sampleWithRequiredData);

        const userPreference = service.getUserPreference(formGroup);

        expect(userPreference).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IUserPreference should not enable id FormControl', () => {
        const formGroup = service.createUserPreferenceFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewUserPreference should disable id FormControl', () => {
        const formGroup = service.createUserPreferenceFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
