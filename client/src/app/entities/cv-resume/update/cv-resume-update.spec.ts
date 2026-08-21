import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { ICvResume } from '../cv-resume.model';
import { CvResumeService } from '../service/cv-resume.service';

import { CvResumeFormService } from './cv-resume-form.service';
import { CvResumeUpdate } from './cv-resume-update';

describe('CvResume Management Update Component', () => {
  let comp: CvResumeUpdate;
  let fixture: ComponentFixture<CvResumeUpdate>;
  let activatedRoute: ActivatedRoute;
  let cvResumeFormService: CvResumeFormService;
  let cvResumeService: CvResumeService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(CvResumeUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    cvResumeFormService = TestBed.inject(CvResumeFormService);
    cvResumeService = TestBed.inject(CvResumeService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const cvResume: ICvResume = { id: 15106 };

      activatedRoute.data = of({ cvResume });
      comp.ngOnInit();

      expect(comp.cvResume).toEqual(cvResume);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResume>();
      const cvResume = { id: 8461 };
      vitest.spyOn(cvResumeFormService, 'getCvResume').mockReturnValue(cvResume);
      vitest.spyOn(cvResumeService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResume });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(cvResume);
      saveSubject.complete();

      // THEN
      expect(cvResumeFormService.getCvResume).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(cvResumeService.update).toHaveBeenCalledWith(expect.objectContaining(cvResume));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResume>();
      const cvResume = { id: 8461 };
      vitest.spyOn(cvResumeFormService, 'getCvResume').mockReturnValue({ id: null });
      vitest.spyOn(cvResumeService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResume: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(cvResume);
      saveSubject.complete();

      // THEN
      expect(cvResumeFormService.getCvResume).toHaveBeenCalled();
      expect(cvResumeService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResume>();
      const cvResume = { id: 8461 };
      vitest.spyOn(cvResumeService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResume });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(cvResumeService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
