import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../cv-resume-version.test-samples';

import { CvResumeVersionFormService } from './cv-resume-version-form.service';

describe('CvResumeVersion Form Service', () => {
  let service: CvResumeVersionFormService;

  beforeEach(() => {
    service = TestBed.inject(CvResumeVersionFormService);
  });

  describe('Service methods', () => {
    describe('createCvResumeVersionFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createCvResumeVersionFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            versionNumber: expect.any(Object),
            title: expect.any(Object),
            template: expect.any(Object),
            data: expect.any(Object),
            createdAt: expect.any(Object),
            resume: expect.any(Object),
          }),
        );
      });

      it('passing ICvResumeVersion should create a new form with FormGroup', () => {
        const formGroup = service.createCvResumeVersionFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            versionNumber: expect.any(Object),
            title: expect.any(Object),
            template: expect.any(Object),
            data: expect.any(Object),
            createdAt: expect.any(Object),
            resume: expect.any(Object),
          }),
        );
      });
    });

    describe('getCvResumeVersion', () => {
      it('should return NewCvResumeVersion for default CvResumeVersion initial value', () => {
        const formGroup = service.createCvResumeVersionFormGroup(sampleWithNewData);

        const cvResumeVersion = service.getCvResumeVersion(formGroup);

        expect(cvResumeVersion).toMatchObject(sampleWithNewData);
      });

      it('should return NewCvResumeVersion for empty CvResumeVersion initial value', () => {
        const formGroup = service.createCvResumeVersionFormGroup();

        const cvResumeVersion = service.getCvResumeVersion(formGroup);

        expect(cvResumeVersion).toMatchObject({});
      });

      it('should return ICvResumeVersion', () => {
        const formGroup = service.createCvResumeVersionFormGroup(sampleWithRequiredData);

        const cvResumeVersion = service.getCvResumeVersion(formGroup);

        expect(cvResumeVersion).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing ICvResumeVersion should not enable id FormControl', () => {
        const formGroup = service.createCvResumeVersionFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewCvResumeVersion should disable id FormControl', () => {
        const formGroup = service.createCvResumeVersionFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
